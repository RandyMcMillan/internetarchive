use serde_json::{Map, Value};

/// Archive.org account details.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Account {
    pub locked: bool,
    pub verified: bool,
    pub email: String,
    pub canonical_email: String,
    pub itemname: String,
    pub screenname: String,
    pub notifications: Vec<String>,
    pub has_disability_access: bool,
    pub lastlogin: String,
    pub createdate: String,
}

impl Account {
    /// Build an account from JSON data.
    pub fn from_json(json_data: &Map<String, Value>) -> Result<Self, String> {
        macro_rules! req {
            ($key:literal) => {
                json_data
                    .get($key)
                    .ok_or_else(|| format!("Missing required field in JSON data: {}", $key))?
            };
        }

        let notifications = req!("notifications")
            .as_array()
            .ok_or_else(|| "notifications must be an array".to_string())?
            .iter()
            .filter_map(Value::as_str)
            .map(str::to_string)
            .collect();

        Ok(Self {
            locked: req!("locked").as_bool().unwrap_or(false),
            verified: req!("verified").as_bool().unwrap_or(false),
            email: req!("email").as_str().unwrap_or_default().to_string(),
            canonical_email: req!("canonical_email")
                .as_str()
                .unwrap_or_default()
                .to_string(),
            itemname: req!("itemname").as_str().unwrap_or_default().to_string(),
            screenname: req!("screenname").as_str().unwrap_or_default().to_string(),
            notifications,
            has_disability_access: req!("has_disability_access").as_bool().unwrap_or(false),
            lastlogin: req!("lastlogin").as_str().unwrap_or_default().to_string(),
            createdate: req!("createdate").as_str().unwrap_or_default().to_string(),
        })
    }

    /// Convert the account to a JSON-like map.
    pub fn to_json(&self) -> Map<String, Value> {
        let mut out = Map::new();
        out.insert("locked".to_string(), Value::Bool(self.locked));
        out.insert("verified".to_string(), Value::Bool(self.verified));
        out.insert("email".to_string(), Value::String(self.email.clone()));
        out.insert(
            "canonical_email".to_string(),
            Value::String(self.canonical_email.clone()),
        );
        out.insert("itemname".to_string(), Value::String(self.itemname.clone()));
        out.insert("screenname".to_string(), Value::String(self.screenname.clone()));
        out.insert(
            "notifications".to_string(),
            Value::Array(
                self.notifications
                    .iter()
                    .cloned()
                    .map(Value::String)
                    .collect(),
            ),
        );
        out.insert(
            "has_disability_access".to_string(),
            Value::Bool(self.has_disability_access),
        );
        out.insert("lastlogin".to_string(), Value::String(self.lastlogin.clone()));
        out.insert("createdate".to_string(), Value::String(self.createdate.clone()));
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_account_json() {
        let json_data = serde_json::json!({
            "locked": false,
            "verified": true,
            "email": "foo@example.com",
            "canonical_email": "foo@example.com",
            "itemname": "@foo",
            "screenname": "Foo",
            "notifications": ["a", "b"],
            "has_disability_access": false,
            "lastlogin": "2026-01-01",
            "createdate": "2026-01-01"
        });
        let account = Account::from_json(json_data.as_object().unwrap()).unwrap();
        assert_eq!(account.screenname, "Foo");
        assert_eq!(account.notifications, vec!["a", "b"]);
        assert_eq!(account.to_json().get("email"), Some(&Value::String("foo@example.com".into())));
    }
}
