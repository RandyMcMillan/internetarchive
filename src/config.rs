use std::collections::BTreeMap;
use std::env;
use std::fmt::{Display, Formatter};
use std::fs;
use std::path::{Path, PathBuf};

/// A recursive configuration value used to merge nested config maps.
#[derive(Debug, Clone, PartialEq)]
pub enum ConfigValue {
    Null,
    Bool(bool),
    Integer(i64),
    Float(f64),
    String(String),
    List(Vec<ConfigValue>),
    Map(ConfigMap),
}

/// The nested configuration shape used by the Rust core.
pub type ConfigMap = BTreeMap<String, ConfigValue>;

/// Result of resolving a configuration file path.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigFileResolution {
    pub path: PathBuf,
    pub is_xdg: bool,
}

/// Errors raised while loading or merging configuration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfigError {
    /// Only one of the Archive.org S3 environment variables was present.
    MissingPairedEnvironmentVariable,
    /// The configuration file could not be read.
    Io(String),
}

impl Display for ConfigError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingPairedEnvironmentVariable => write!(
                f,
                "Both IA_ACCESS_KEY_ID and IA_SECRET_ACCESS_KEY environment variables must be set together, or neither should be set."
            ),
            Self::Io(msg) => write!(f, "{msg}"),
        }
    }
}

impl std::error::Error for ConfigError {}

/// Recursively merge `updates` into `target`.
pub fn deep_update(target: &mut ConfigMap, updates: &ConfigMap) {
    for (key, value) in updates {
        match (target.get_mut(key), value) {
            (Some(ConfigValue::Map(existing)), ConfigValue::Map(incoming)) => {
                deep_update(existing, incoming);
            }
            _ => {
                target.insert(key.clone(), value.clone());
            }
        }
    }
}

/// Resolve the configuration file path using the Python search order.
pub fn parse_config_file_path(config_file: Option<&Path>) -> ConfigFileResolution {
    if let Some(config_file) = config_file {
        return ConfigFileResolution {
            path: config_file.to_path_buf(),
            is_xdg: false,
        };
    }

    let xdg_config_home = env::var_os("XDG_CONFIG_HOME")
        .filter(|value| Path::new(value).is_absolute())
        .map(PathBuf::from)
        .unwrap_or_else(|| home_dir().join(".config"));
    let xdg_config_file = xdg_config_home.join("internetarchive").join("ia.ini");

    let candidates = [
        env::var_os("IA_CONFIG_FILE").map(PathBuf::from),
        Some(xdg_config_file.clone()),
        Some(home_dir().join(".config").join("ia.ini")),
        Some(home_dir().join(".ia")),
    ];

    for candidate in candidates.into_iter().flatten() {
        if candidate.is_file() {
            let is_xdg = candidate == xdg_config_file;
            return ConfigFileResolution {
                path: candidate,
                is_xdg,
            };
        }
    }

    ConfigFileResolution {
        path: env::var_os("IA_CONFIG_FILE")
            .map(PathBuf::from)
            .unwrap_or(xdg_config_file),
        is_xdg: false,
    }
}

/// Parse a minimal INI-style config file into a nested config map.
pub fn parse_config_file(
    config_file: Option<&Path>,
) -> Result<(ConfigFileResolution, ConfigMap), ConfigError> {
    let resolution = parse_config_file_path(config_file);
    let mut config = ConfigMap::new();

    if resolution.path.is_file() {
        let contents = fs::read_to_string(&resolution.path)
            .map_err(|err| ConfigError::Io(err.to_string()))?;
        config = parse_ini_contents(&contents);
    }

    ensure_defaults(&mut config);
    Ok((resolution, config))
}

/// Merge a config file, environment, and an optional config override.
pub fn get_config(
    config: Option<ConfigMap>,
    config_file: Option<&Path>,
) -> Result<ConfigMap, ConfigError> {
    let (_resolution, mut merged) = parse_config_file(config_file)?;

    let env_access_key = env::var("IA_ACCESS_KEY_ID").ok();
    let env_secret_key = env::var("IA_SECRET_ACCESS_KEY").ok();
    if env_access_key.is_some() ^ env_secret_key.is_some() {
        return Err(ConfigError::MissingPairedEnvironmentVariable);
    }
    if let (Some(access), Some(secret)) = (env_access_key, env_secret_key) {
        merged.insert(
            "s3".to_string(),
            ConfigValue::Map(ConfigMap::from([
                ("access".to_string(), ConfigValue::String(access)),
                ("secret".to_string(), ConfigValue::String(secret)),
            ])),
        );
    }

    if let Some(config) = config {
        deep_update(&mut merged, &config);
    }

    Ok(merged)
}

fn ensure_defaults(config: &mut ConfigMap) {
    config
        .entry("s3".to_string())
        .or_insert_with(|| {
            ConfigValue::Map(ConfigMap::from([
                ("access".to_string(), ConfigValue::Null),
                ("secret".to_string(), ConfigValue::Null),
            ]))
        });
    config
        .entry("cookies".to_string())
        .or_insert_with(|| {
            ConfigValue::Map(ConfigMap::from([
                ("logged-in-user".to_string(), ConfigValue::Null),
                ("logged-in-sig".to_string(), ConfigValue::Null),
            ]))
        });
    config.entry("general".to_string()).or_insert_with(|| {
        ConfigValue::Map(ConfigMap::from([(
            "screenname".to_string(),
            ConfigValue::Null,
        )]))
    });
}

fn parse_ini_contents(contents: &str) -> ConfigMap {
    let mut config = ConfigMap::new();
    let mut current_section = String::new();

    for line in contents.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') || line.starts_with(';') {
            continue;
        }
        if let Some(section) = line.strip_prefix('[').and_then(|line| line.strip_suffix(']')) {
            current_section = section.trim().to_string();
            config
                .entry(current_section.clone())
                .or_insert_with(|| ConfigValue::Map(ConfigMap::new()));
            continue;
        }

        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        let key = key.trim().to_string();
        let value = value.trim();
        let entry = config
            .entry(current_section.clone())
            .or_insert_with(|| ConfigValue::Map(ConfigMap::new()));
        if let ConfigValue::Map(section) = entry {
            section.insert(key, parse_config_value(&current_section, value));
        }
    }

    config
}

fn parse_config_value(section: &str, value: &str) -> ConfigValue {
    if section == "general" && value.eq_ignore_ascii_case("true") {
        ConfigValue::Bool(true)
    } else if section == "general" && value.eq_ignore_ascii_case("false") {
        ConfigValue::Bool(false)
    } else {
        ConfigValue::String(value.to_string())
    }
}

fn home_dir() -> PathBuf {
    env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn merges_nested_maps_recursively() {
        let mut target = ConfigMap::from([
            (
                "general".to_string(),
                ConfigValue::Map(ConfigMap::from([(
                    "secure".to_string(),
                    ConfigValue::Bool(true),
                )])),
            ),
            ("s3".to_string(), ConfigValue::String("old".to_string())),
        ]);

        let updates = ConfigMap::from([
            (
                "general".to_string(),
                ConfigValue::Map(ConfigMap::from([(
                    "host".to_string(),
                    ConfigValue::String("archive.org".to_string()),
                )])),
            ),
            ("s3".to_string(), ConfigValue::String("new".to_string())),
        ]);

        deep_update(&mut target, &updates);

        assert_eq!(
            target.get("general"),
            Some(&ConfigValue::Map(ConfigMap::from([
                ("secure".to_string(), ConfigValue::Bool(true)),
                (
                    "host".to_string(),
                    ConfigValue::String("archive.org".to_string())
                ),
            ])))
        );
        assert_eq!(
            target.get("s3"),
            Some(&ConfigValue::String("new".to_string()))
        );
    }

    #[test]
    fn parses_ini_contents_and_adds_defaults() {
        let contents = "[general]\nsecure = false\n";
        let config = parse_ini_contents(contents);
        assert!(matches!(
            config.get("general"),
            Some(ConfigValue::Map(section)) if section.get("secure") == Some(&ConfigValue::Bool(false))
        ));
        let mut config = config;
        ensure_defaults(&mut config);
        assert!(config.contains_key("s3"));
        assert!(config.contains_key("cookies"));
    }

    #[test]
    fn merges_environment_config() {
        let config = get_config(None, None);
        assert!(config.is_ok());
    }
}
