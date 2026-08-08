use std::collections::BTreeMap;

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
}
