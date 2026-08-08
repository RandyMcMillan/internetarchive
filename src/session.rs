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
}
