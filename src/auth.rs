use std::collections::BTreeMap;

use crate::errors::AuthenticationError;

/// IA-S3 request authentication.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct S3Auth {
    pub access_key: Option<String>,
    pub secret_key: Option<String>,
}

impl S3Auth {
    /// Create a new S3 auth helper.
    pub fn new(access_key: Option<String>, secret_key: Option<String>) -> Self {
        Self {
            access_key,
            secret_key,
        }
    }

    /// Apply the `Authorization` header.
    pub fn apply(&self, headers: &mut BTreeMap<String, String>) -> Result<(), AuthenticationError> {
        match (&self.access_key, &self.secret_key) {
            (Some(access), Some(secret)) => {
                headers.insert("Authorization".to_string(), format!("LOW {access}:{secret}"));
                Ok(())
            }
            (None, Some(_)) => Err(AuthenticationError::MissingAccessKey),
            (Some(_), None) => Err(AuthenticationError::MissingSecretKey),
            (None, None) => Err(AuthenticationError::MissingAccessAndSecretKey),
        }
    }
}

/// IA-S3 metadata write authentication.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct S3PostAuth {
    pub access_key: Option<String>,
    pub secret_key: Option<String>,
}

impl S3PostAuth {
    /// Create a new post auth helper.
    pub fn new(access_key: Option<String>, secret_key: Option<String>) -> Self {
        Self {
            access_key,
            secret_key,
        }
    }

    /// Append credentials to a URL-encoded request body.
    pub fn apply(&self, body: &mut String) {
        body.push_str(&format!(
            "&access={}&secret={}",
            self.access_key.as_deref().unwrap_or(""),
            self.secret_key.as_deref().unwrap_or("")
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn applies_low_auth_header() {
        let auth = S3Auth::new(Some("a".to_string()), Some("s".to_string()));
        let mut headers = BTreeMap::new();
        auth.apply(&mut headers).unwrap();
        assert_eq!(headers.get("Authorization"), Some(&"LOW a:s".to_string()));
    }

    #[test]
    fn rejects_missing_auth_parts() {
        let auth = S3Auth::new(None, None);
        assert!(matches!(
            auth.apply(&mut BTreeMap::new()),
            Err(AuthenticationError::MissingAccessAndSecretKey)
        ));
    }
}
