/// Search request state used by the core crate.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchQuery {
    pub query: String,
    pub fields: Vec<String>,
    pub sorts: Vec<String>,
    pub full_text_search: bool,
    pub dsl_fts: bool,
}

impl SearchQuery {
    /// Construct a new search query.
    pub fn new(query: impl Into<String>) -> Self {
        Self {
            query: query.into(),
            fields: Vec::new(),
            sorts: Vec::new(),
            full_text_search: false,
            dsl_fts: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_search_query() {
        let query = SearchQuery::new("nasa");
        assert_eq!(query.query, "nasa");
        assert!(!query.full_text_search);
    }
}
