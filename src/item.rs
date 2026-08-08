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
}

impl ArchiveItem {
    /// Construct a new item descriptor.
    pub fn new(identifier: impl Into<String>) -> Self {
        Self {
            identifier: identifier.into(),
            title: None,
            mediatype: None,
            exists: false,
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
