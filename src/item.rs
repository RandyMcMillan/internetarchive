use glob::Pattern;
use serde_json::{Map, Value};

use crate::utils::flatten_pipe_patterns;

/// High-level item kind used by the core crate.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ArchiveItemKind {
    Item,
    Collection,
}

/// Minimal item descriptor for Archive.org entities.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArchiveItem {
    pub identifier: String,
    pub title: Option<String>,
    pub mediatype: Option<String>,
    pub exists: bool,
    pub files: Vec<ArchiveFile>,
}

impl ArchiveItem {
    /// Construct a new item descriptor.
    pub fn new(identifier: impl Into<String>) -> Self {
        Self {
            identifier: identifier.into(),
            title: None,
            mediatype: None,
            exists: false,
            files: Vec::new(),
        }
    }

    /// Construct an item descriptor from Archive.org metadata JSON.
    pub fn from_metadata(item_metadata: &Map<String, Value>) -> Self {
        let metadata = item_metadata
            .get("metadata")
            .and_then(Value::as_object)
            .cloned()
            .unwrap_or_default();
        let files = item_metadata
            .get("files")
            .and_then(Value::as_array)
            .map(|entries| {
                entries
                    .iter()
                    .filter_map(|entry| ArchiveFile::from_metadata(entry).ok())
                    .collect()
            })
            .unwrap_or_default();

        let identifier = metadata
            .get("identifier")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string();
        let title = metadata
            .get("title")
            .and_then(Value::as_str)
            .map(str::to_string);
        let mediatype = metadata
            .get("mediatype")
            .and_then(Value::as_str)
            .map(str::to_string);

        Self {
            identifier,
            title,
            mediatype,
            exists: !item_metadata.is_empty(),
            files,
        }
    }

    /// Return the item kind from the stored mediatype.
    pub fn kind(&self) -> ArchiveItemKind {
        if self.mediatype.as_deref() == Some("collection") {
            ArchiveItemKind::Collection
        } else {
            ArchiveItemKind::Item
        }
    }

    /// Return the files matching the Python `Item.get_files()` selection rules.
    pub fn matching_files(
        &self,
        files: &[String],
        formats: &[String],
        glob_pattern: &[String],
        exclude_pattern: &[String],
        on_the_fly: bool,
    ) -> Vec<ArchiveFile> {
        let mut item_files = self.files.clone();
        if on_the_fly {
            item_files.extend([
                ArchiveFile::new(self.identifier.clone(), format!("{}.epub", self.identifier)),
                ArchiveFile::new(self.identifier.clone(), format!("{}.mobi", self.identifier)),
                ArchiveFile::new(
                    self.identifier.clone(),
                    format!("{}_daisy.zip", self.identifier),
                ),
                ArchiveFile::new(
                    self.identifier.clone(),
                    format!("{}_archive_marc.xml", self.identifier),
                ),
            ]);
        }

        let has_any_filter = !files.is_empty() || !formats.is_empty() || !glob_pattern.is_empty();
        if !has_any_filter {
            return item_files;
        }

        let patterns = flatten_pipe_patterns(glob_pattern);
        let exclude_patterns = flatten_pipe_patterns(exclude_pattern);

        item_files
            .into_iter()
            .filter(|file| {
                let name = file.name.as_str();
                if files.iter().any(|candidate| candidate == name) {
                    return true;
                }
                if formats.iter().any(|candidate| file.format.as_deref() == Some(candidate)) {
                    return true;
                }
                if patterns.is_empty() {
                    return false;
                }

                let included = patterns
                    .iter()
                    .any(|pattern| Pattern::new(pattern).is_ok_and(|p| p.matches(name)));
                if !included {
                    return false;
                }

                !exclude_patterns
                    .iter()
                    .any(|pattern| Pattern::new(pattern).is_ok_and(|p| p.matches(name)))
            })
            .collect()
    }

    /// Return `true` if a file matching the provided name exists.
    pub fn has_file(&self, name: &str) -> bool {
        self.files.iter().any(|file| file.name == name)
    }
}

/// Minimal file descriptor for Archive.org item files.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArchiveFile {
    pub identifier: String,
    pub name: String,
    pub size: Option<u64>,
    pub format: Option<String>,
    pub source: Option<String>,
    pub md5: Option<String>,
    pub sha1: Option<String>,
    pub crc32: Option<String>,
}

impl ArchiveFile {
    /// Construct a new file descriptor.
    pub fn new(identifier: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            identifier: identifier.into(),
            name: name.into(),
            size: None,
            format: None,
            source: None,
            md5: None,
            sha1: None,
            crc32: None,
        }
    }

    /// Construct a file descriptor from Archive.org file metadata JSON.
    pub fn from_metadata(file_metadata: &Value) -> Result<Self, &'static str> {
        let metadata = file_metadata
            .as_object()
            .ok_or("file metadata must be an object")?;
        let identifier = metadata
            .get("identifier")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string();
        let name = metadata
            .get("name")
            .and_then(Value::as_str)
            .ok_or("file metadata missing name")?
            .to_string();
        let size = metadata.get("size").and_then(Value::as_u64);
        let format = metadata
            .get("format")
            .and_then(Value::as_str)
            .map(str::to_string);
        let source = metadata
            .get("source")
            .and_then(Value::as_str)
            .map(str::to_string);
        let md5 = metadata
            .get("md5")
            .and_then(Value::as_str)
            .map(str::to_string);
        let sha1 = metadata
            .get("sha1")
            .and_then(Value::as_str)
            .map(str::to_string);
        let crc32 = metadata
            .get("crc32")
            .and_then(Value::as_str)
            .map(str::to_string);

        Ok(Self {
            identifier,
            name,
            size,
            format,
            source,
            md5,
            sha1,
            crc32,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_collections() {
        let mut item = ArchiveItem::new("nasa");
        item.mediatype = Some("collection".to_string());
        assert_eq!(item.kind(), ArchiveItemKind::Collection);
    }

    #[test]
    fn loads_metadata_and_files() {
        let item_metadata = serde_json::json!({
            "metadata": {
                "identifier": "nasa",
                "title": "NASA",
                "mediatype": "collection"
            },
            "files": [
                {"identifier": "nasa", "name": "nasa_meta.xml", "format": "Metadata"}
            ]
        });
        let item = ArchiveItem::from_metadata(item_metadata.as_object().unwrap());
        assert_eq!(item.identifier, "nasa");
        assert_eq!(item.title.as_deref(), Some("NASA"));
        assert_eq!(item.kind(), ArchiveItemKind::Collection);
        assert!(item.has_file("nasa_meta.xml"));
    }
}
