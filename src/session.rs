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
}
