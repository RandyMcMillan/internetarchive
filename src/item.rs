use glob::Pattern;

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
}
