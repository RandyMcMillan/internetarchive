use std::fs;
use std::io::{Read, Seek, SeekFrom};
use std::path::MAIN_SEPARATOR;

use serde_json::{Map, Value};

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

/// Recursively remove matching values from a JSON-like value.
pub fn delete_items_from_dict(value: Value, to_delete: &str) -> Value {
        match value {
            Value::Object(map) => {
                let mut result = Map::new();
                for (key, child) in map {
                    let child = delete_items_from_dict(child, to_delete);
                    if !child.is_null() {
                        result.insert(key, child);
                    }
                }
                Value::Object(result)
            }
            Value::Array(items) => Value::Array(
                items
                    .into_iter()
                    .map(|child| delete_items_from_dict(child, to_delete))
                    .filter(|child| {
                        !child.is_null() && child != &Value::String(to_delete.to_string())
                    })
                    .collect(),
            ),
            other if other == Value::String(to_delete.to_string()) => Value::Null,
            other => other,
        }
}

/// Recursively drop null-like values from a JSON-like value.
pub fn remove_none(value: Value) -> Value {
        match value {
            Value::Object(map) => {
                let mut result = Map::new();
                for (key, child) in map {
                    let child = remove_none(child);
                    if !child.is_null() {
                        result.insert(key, child);
                    }
                }
                Value::Object(result)
            }
            Value::Array(items) => Value::Array(
                items
                    .into_iter()
                    .map(remove_none)
                    .filter(|value| !value.is_null())
                    .collect(),
            ),
            other => other,
        }
}

/// Calculate the MD5 checksum for a readable stream.
pub fn get_md5<R: Read + Seek>(reader: &mut R) -> std::io::Result<String> {
    let mut context = md5::Context::new();
    let mut buffer = [0_u8; 8192];

    loop {
        let read = reader.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        context.consume(&buffer[..read]);
    }

    reader.seek(SeekFrom::Start(0))?;
    Ok(format!("{:x}", context.compute()))
}

/// Return the size of a file on disk.
pub fn get_file_size(path: impl AsRef<std::path::Path>) -> std::io::Result<u64> {
    Ok(fs::metadata(path)?.len())
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

    #[test]
    fn removes_matching_values() {
        let value = serde_json::json!({"a":"REMOVE_TAG","b":[1,"REMOVE_TAG",2]});
        assert_eq!(
            delete_items_from_dict(value, "REMOVE_TAG"),
            serde_json::json!({"b":[1,2]})
        );
    }

    #[test]
    fn calculates_md5_and_resets_reader() {
        use std::io::Cursor;

        let mut cursor = Cursor::new(b"hello".to_vec());
        assert_eq!(get_md5(&mut cursor).unwrap(), "5d41402abc4b2a76b9719d911017c592");
        assert_eq!(cursor.position(), 0);
    }
}
