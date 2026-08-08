use std::fs;
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf, MAIN_SEPARATOR};
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
    let to_delete_value = Value::String(to_delete.to_string());
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
                .filter(|child| !child.is_null() && child != &to_delete_value)
                .collect(),
        ),
        other if other == to_delete_value => Value::Null,
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

/// Check whether the current platform is Windows.
pub fn is_windows() -> bool {
    cfg!(windows)
}

/// Validate an Archive.org metadata key.
pub fn is_valid_metadata_key(name: &str) -> bool {
    let Some((base, suffix)) = name.split_once('[') else {
        return is_valid_metadata_key_base(name);
    };
    suffix.ends_with(']') && is_valid_metadata_key_base(base)
}

fn is_valid_metadata_key_base(name: &str) -> bool {
    let Some(first) = name.chars().next() else {
        return false;
    };
    if !first.is_ascii_alphabetic() {
        return false;
    }
    name.chars()
        .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '.' | '-' | '_'))
}

/// Merge two dictionaries represented as JSON objects.
pub fn merge_dictionaries(
    dict0: Option<Map<String, Value>>,
    dict1: Option<Map<String, Value>>,
    keys_to_drop: Option<&[String]>,
) -> Map<String, Value> {
    let mut result = dict0.unwrap_or_default();
    if let Some(keys_to_drop) = keys_to_drop {
        for key in keys_to_drop {
            result.remove(key);
        }
    }
    if let Some(dict1) = dict1 {
        for (key, value) in dict1 {
            result.insert(key, value);
        }
    }
    result
}

/// Parse a cookie string into a structured map.
pub fn parse_dict_cookies(value: &str) -> Map<String, Value> {
    let mut result = Map::new();
    for item in value.split(';') {
        let item = item.trim();
        if item.is_empty() {
            continue;
        }
        let Some((name, cookie_value)) = item.split_once('=') else {
            result.insert(item.to_string(), Value::Null);
            continue;
        };
        result.insert(
            name.trim().to_string(),
            Value::String(cookie_value.trim().to_string()),
        );
    }
    result
        .entry("domain".to_string())
        .or_insert_with(|| Value::String(".archive.org".to_string()));
    result
        .entry("path".to_string())
        .or_insert_with(|| Value::String("/".to_string()));
    result
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

/// Yield file paths and relative keys for all files under a directory.
pub fn iter_directory(directory: &Path) -> std::io::Result<Vec<(PathBuf, PathBuf)>> {
    let mut items = Vec::new();
    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            items.extend(iter_directory(&path)?);
        } else {
            let key = path
                .strip_prefix(directory)
                .unwrap_or(&path)
                .to_path_buf();
            items.push((path, key));
        }
    }
    Ok(items)
}

/// Count files and total size recursively.
pub fn recursive_file_count_and_size(
    files: &[impl AsRef<Path>],
) -> std::io::Result<(usize, u64)> {
    let mut total_files = 0;
    let mut total_size = 0;
    for file in files {
        let path = file.as_ref();
        if path.is_dir() {
            for (item_path, _) in iter_directory(path)? {
                total_files += 1;
                total_size += get_file_size(item_path)?;
            }
        } else {
            total_files += 1;
            total_size += get_file_size(path)?;
        }
    }
    Ok((total_files, total_size))
}

/// Count files recursively.
pub fn recursive_file_count(files: &[impl AsRef<Path>]) -> std::io::Result<usize> {
    recursive_file_count_and_size(files).map(|(count, _)| count)
}

/// Return true when a value is directory-like.
pub fn is_dir(path: impl AsRef<Path>) -> bool {
    path.as_ref().is_dir()
}

/// Return true when a value is not path-like and should be treated as file-like.
pub fn is_filelike_obj<T: ?Sized>(_: &T) -> bool {
    false
}

fn encode_percent(byte: u8) -> String {
    format!("%{byte:02X}")
}

fn encode_char(ch: char) -> String {
    let mut encoded = String::new();
    let mut buf = [0_u8; 4];
    for byte in ch.encode_utf8(&mut buf).as_bytes() {
        encoded.push_str(&encode_percent(*byte));
    }
    encoded
}

/// Sanitize a filename for Windows downloads.
pub fn sanitize_windows_filename(name: &str) -> (String, bool) {
    if name.is_empty() {
        return (name.to_string(), false);
    }

    let reserved = [
        "CON", "PRN", "AUX", "NUL", "COM1", "COM2", "COM3", "COM4", "COM5", "COM6",
        "COM7", "COM8", "COM9", "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6",
        "LPT7", "LPT8", "LPT9",
    ];
    let upper_name = name.to_ascii_uppercase();
    let mut encode_indexes = std::collections::BTreeSet::new();

    for (idx, ch) in name.chars().enumerate() {
        if ch.is_control() || matches!(ch, '<' | '>' | ':' | '"' | '\\' | '|' | '?' | '*') {
            encode_indexes.insert(idx);
        }
    }

    for (idx, ch) in name.char_indices().rev() {
        if matches!(ch, ' ' | '.') {
            encode_indexes.insert(idx);
        } else {
            break;
        }
    }

    for base in reserved {
        if upper_name == base || upper_name.starts_with(&format!("{base}.")) {
            encode_indexes.insert(base.len() - 1);
            break;
        }
    }

    if encode_indexes.is_empty() {
        return (name.to_string(), false);
    }

    let mut out = String::new();
    for (idx, ch) in name.chars().enumerate() {
        if ch == '%' {
            out.push_str("%25");
        } else if encode_indexes.contains(&idx) {
            out.push_str(&encode_char(ch));
        } else {
            out.push(ch);
        }
    }
    let modified = out != name;
    (out, modified)
}

/// Return true if target_path resolves inside base_dir.
pub fn is_path_within_directory(base_dir: &str, target_path: &str) -> bool {
    let base_real = std::fs::canonicalize(base_dir).unwrap_or_else(|_| Path::new(base_dir).to_path_buf());
    let target_real = std::fs::canonicalize(target_path).unwrap_or_else(|_| Path::new(target_path).to_path_buf());
    target_real.starts_with(base_real)
}

/// Sanitize a relative path for Windows downloads.
pub fn sanitize_windows_relpath(rel_path: &str, verbose: bool) -> (String, bool) {
    if !is_windows() || rel_path.is_empty() {
        return (rel_path.to_string(), false);
    }
    let mut modified = false;
    let mut parts = Vec::new();
    for part in rel_path.split('/') {
        let (sanitized, changed) = sanitize_windows_filename(part);
        modified |= changed;
        parts.push(sanitized);
    }
    let joined = parts.join(std::path::MAIN_SEPARATOR_STR);
    let _ = verbose;
    (joined, modified)
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

    #[test]
    fn validates_metadata_keys() {
        assert!(is_valid_metadata_key("identifier"));
        assert!(is_valid_metadata_key("index[0]"));
        assert!(!is_valid_metadata_key("invalid key"));
    }

    #[test]
    fn parses_cookie_values() {
        let cookies = parse_dict_cookies("logged-in-user=a%40b; path=/x");
        assert_eq!(
            cookies.get("domain"),
            Some(&Value::String(".archive.org".to_string()))
        );
        assert_eq!(cookies.get("path"), Some(&Value::String("/x".to_string())));
    }

    #[test]
    fn merges_dictionary_values() {
        let mut dict0 = Map::new();
        dict0.insert("a".to_string(), Value::String("1".to_string()));
        let mut dict1 = Map::new();
        dict1.insert("b".to_string(), Value::String("2".to_string()));
        let result = merge_dictionaries(Some(dict0), Some(dict1), None);
        assert_eq!(result.get("a"), Some(&Value::String("1".to_string())));
        assert_eq!(result.get("b"), Some(&Value::String("2".to_string())));
    }

    #[test]
    fn validates_metadata_key_shapes() {
        assert!(is_valid_metadata_key("identifier"));
        assert!(is_valid_metadata_key("index[0]"));
        assert!(!is_valid_metadata_key("invalid key"));
        assert!(!is_valid_metadata_key("_metadata"));
    }

    #[test]
    fn sanitizes_windows_filenames() {
        let (sanitized, modified) = sanitize_windows_filename("AUX.txt");
        assert!(modified);
        assert!(sanitized.contains("%"));
    }

    #[test]
    fn parses_cookie_defaults() {
        let cookies = parse_dict_cookies("logged-in-user=a%40b");
        assert_eq!(
            cookies.get("domain"),
            Some(&Value::String(".archive.org".to_string()))
        );
        assert_eq!(cookies.get("path"), Some(&Value::String("/".to_string())));
    }

    #[test]
    fn counts_files_recursively() {
        let tmp = std::env::temp_dir().join(format!(
            "internetarchive-core-{}",
            std::process::id()
        ));
        let nested = tmp.join("nested");
        let _ = fs::remove_dir_all(&tmp);
        fs::create_dir_all(&nested).unwrap();
        fs::write(tmp.join("a.txt"), b"a").unwrap();
        fs::write(nested.join("b.txt"), b"bb").unwrap();
        let (count, size) = recursive_file_count_and_size(&[tmp.as_path()]).unwrap();
        assert_eq!(count, 2);
        assert_eq!(size, 3);
        let _ = fs::remove_dir_all(&tmp);
    }
}
