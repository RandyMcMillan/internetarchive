use crate::config::{ConfigMap, ConfigValue};

/// Session configuration used by the Rust core.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArchiveSessionConfig {
    pub secure: bool,
    pub host: String,
    pub user_agent_suffix: Option<String>,
    pub access_key: Option<String>,
    pub secret_key: Option<String>,
}

impl Default for ArchiveSessionConfig {
    fn default() -> Self {
        Self {
            secure: true,
            host: "archive.org".to_string(),
            user_agent_suffix: None,
            access_key: None,
            secret_key: None,
        }

        impl ArchiveSessionConfig {
            /// Build a session config from a merged config map.
            pub fn from_config_map(config: &ConfigMap) -> Self {
                let general = config.get("general");
                let s3 = config.get("s3");

                Self {
                    secure: match general {
                        Some(ConfigValue::Map(section)) => match section.get("secure") {
                            Some(ConfigValue::Bool(value)) => *value,
                            Some(ConfigValue::String(value)) => !value.eq_ignore_ascii_case("false"),
                            _ => true,
                        },
                        _ => true,
                    },
                    host: match general {
                        Some(ConfigValue::Map(section)) => match section.get("host") {
                            Some(ConfigValue::String(value)) if !value.is_empty() => value.clone(),
                            _ => "archive.org".to_string(),
                        },
                        _ => "archive.org".to_string(),
                    },
                    user_agent_suffix: match general {
                        Some(ConfigValue::Map(section)) => match section.get("user_agent_suffix") {
                            Some(ConfigValue::String(value)) if !value.is_empty() => {
                                Some(value.clone())
                            }
                            _ => None,
                        },
                        _ => None,
                    },
                    access_key: match s3 {
                        Some(ConfigValue::Map(section)) => match section.get("access") {
                            Some(ConfigValue::String(value)) if !value.is_empty() => Some(value.clone()),
                            _ => None,
                        },
                        _ => None,
                    },
                    secret_key: match s3 {
                        Some(ConfigValue::Map(section)) => match section.get("secret") {
                            Some(ConfigValue::String(value)) if !value.is_empty() => Some(value.clone()),
                            _ => None,
                        },
                        _ => None,
                    },
                }
            }
        }
    }
}

/// Lightweight core session wrapper.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArchiveSession {
    pub config: ArchiveSessionConfig,
}

impl ArchiveSession {
    /// Create a session with the provided configuration.
    pub fn new(config: ArchiveSessionConfig) -> Self {
        let mut config = config;
        if !config.host.contains("archive.org") {
            config.host.push_str(".archive.org");
        }
        Self { config }
    }

    /// Return the URL scheme used by the session.
    pub fn protocol(&self) -> &'static str {
        if self.config.secure { "https:" } else { "http:" }
    }

    /// Return the configured host, normalized to an archive.org host name.
    pub fn host(&self) -> &str {
        &self.config.host
    }

    /// Build a User-Agent string compatible with the Python implementation.
    pub fn user_agent_string(
        &self,
        crate_version: &str,
        os_name: &str,
        arch: &str,
        locale: &str,
        python_version: &str,
    ) -> String {
        let access_key = self.config.access_key.as_deref().unwrap_or("");
        let suffix = self.config.user_agent_suffix.as_deref().unwrap_or("");
        let mut user_agent = format!(
            "internetarchive/{crate_version} ({os_name} {arch}; N; {locale}; {access_key}) Python/{python_version}"
        );
        if !suffix.is_empty() {
            user_agent.push(' ');
            user_agent.push_str(suffix);
        }
        user_agent
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_to_https_archive_org() {
        let session = ArchiveSession::new(ArchiveSessionConfig::default());
        assert_eq!(session.protocol(), "https:");
        assert_eq!(session.host(), "archive.org");
    }

    #[test]
    fn normalizes_non_archive_hosts() {
        let session = ArchiveSession::new(ArchiveSessionConfig {
            host: "localhost".to_string(),
            ..ArchiveSessionConfig::default()
        });
        assert_eq!(session.host(), "localhost.archive.org");
    }

    #[test]
    fn builds_user_agent_string() {
        let session = ArchiveSession::new(ArchiveSessionConfig {
            access_key: Some("ACCESS".to_string()),
            user_agent_suffix: Some("MyApp/1.0".to_string()),
            ..ArchiveSessionConfig::default()
        });
        assert_eq!(
            session.user_agent_string("1.2.3", "Darwin", "x86_64", "en", "3.10.0"),
            "internetarchive/1.2.3 (Darwin x86_64; N; en; ACCESS) Python/3.10.0 MyApp/1.0"
        );
    }

    #[test]
    fn builds_config_from_map() {
        let config = ConfigMap::from([
            (
                "general".to_string(),
                ConfigValue::Map(ConfigMap::from([
                    ("secure".to_string(), ConfigValue::Bool(false)),
                    (
                        "host".to_string(),
                        ConfigValue::String("beta".to_string()),
                    ),
                    (
                        "user_agent_suffix".to_string(),
                        ConfigValue::String("Rust/0.1".to_string()),
                    ),
                ])),
            ),
            (
                "s3".to_string(),
                ConfigValue::Map(ConfigMap::from([
                    ("access".to_string(), ConfigValue::String("AK".to_string())),
                    ("secret".to_string(), ConfigValue::String("SK".to_string())),
                ])),
            ),
        ]);
        let session = ArchiveSessionConfig::from_config_map(&config);
        assert!(!session.secure);
        assert_eq!(session.host, "beta");
        assert_eq!(session.user_agent_suffix.as_deref(), Some("Rust/0.1"));
        assert_eq!(session.access_key.as_deref(), Some("AK"));
        assert_eq!(session.secret_key.as_deref(), Some("SK"));
    }
}
