use std::path::MAIN_SEPARATOR;

/// Normalize a path into Archive.org-style forward-slash form with a leading `/`.
pub fn norm_filepath(path: impl AsRef<str>) -> String {
    let mut normalized = path.as_ref().replace(MAIN_SEPARATOR, "/");
    if !normalized.starts_with('/') {
        normalized.insert(0, '/');
    }
    normalized
}

/// Return true when a string must be quoted for HTTP header use.
pub fn needs_quote(value: &str) -> bool {
    !value.is_ascii() || value.chars().any(char::is_whitespace)
}

/// Flatten a list of pipe-delimited patterns into a single list.
pub fn flatten_pipe_patterns<S: AsRef<str>>(patterns: &[S]) -> Vec<String> {
    patterns
        .iter()
        .flat_map(|pattern| pattern.as_ref().split('|').map(str::to_owned))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_paths() {
        assert_eq!(norm_filepath("foo/bar"), "/foo/bar");
    }

    #[test]
    fn detects_quote_need() {
        assert!(needs_quote("hello world"));
        assert!(needs_quote("føø"));
        assert!(!needs_quote("hello-world"));
    }

    #[test]
    fn flattens_pipe_patterns() {
        let patterns = ["*.jpg|*.xml", "*.torrent"];
        assert_eq!(
            flatten_pipe_patterns(&patterns),
            vec!["*.jpg", "*.xml", "*.torrent"]
        );
    }
}
