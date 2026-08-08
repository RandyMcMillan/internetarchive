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

    /// Set the query fields.
    pub fn with_fields(mut self, fields: Vec<String>) -> Self {
        self.fields = fields;
        self
    }

    /// Set the sort order values.
    pub fn with_sorts(mut self, sorts: Vec<String>) -> Self {
        self.sorts = sorts;
        self
    }

    /// Enable full-text search.
    pub fn with_full_text_search(mut self, dsl_fts: bool) -> Self {
        self.full_text_search = true;
        self.dsl_fts = dsl_fts;
        self
    }

    /// Build the effective query string used by the Python `Search` class.
    pub fn effective_query(&self) -> String {
        if self.full_text_search && !self.dsl_fts {
            format!("!L {}", self.query)
        } else {
            self.query.clone()
        }
    }

    /// Build advanced-search-style query params.
    pub fn advanced_search_params(&self) -> Vec<(String, String)> {
        let mut params = vec![("q".to_string(), self.effective_query())];
        let mut fields = self.fields.clone();
        if fields.iter().all(|field| field != "identifier") {
            fields.push("identifier".to_string());
        }
        for (index, field) in fields.iter().enumerate() {
            params.push((format!("fl[{index}]"), field.clone()));
        }
        for (index, sort) in self.sorts.iter().enumerate() {
            params.push((format!("sort[{index}]"), sort.clone()));
        }
        params.push(("output".to_string(), "json".to_string()));
        params
    }

    /// Build the initial scrape request params.
    pub fn scrape_params(&self) -> Vec<(String, String)> {
        let mut params = vec![("q".to_string(), self.effective_query())];
        if !self.fields.is_empty() {
            params.push(("fields".to_string(), self.fields.join(",")));
        }
        if !self.sorts.is_empty() {
            params.push(("sorts".to_string(), self.sorts.join(",")));
        }
        params
    }

    /// Build the JSON payload used by the full-text search endpoint.
    pub fn fts_payload(&self, size: Option<usize>, scope: Option<&str>) -> Vec<(String, String)> {
        let mut payload = vec![
            ("q".to_string(), self.effective_query()),
            ("size".to_string(), size.unwrap_or(10_000).to_string()),
            ("from".to_string(), "0".to_string()),
            (
                "scroll".to_string(),
                if size.is_some() {
                    "false".to_string()
                } else {
                    "true".to_string()
                },
            ),
        ];
        if let Some(scope) = scope {
            payload.push(("scope".to_string(), scope.to_string()));
        }
        payload
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_search_query() {
        let mut query = SearchQuery::new("nasa");
        query.fields.push("title".to_string());
        query.sorts.push("downloads desc".to_string());
        assert_eq!(query.query, "nasa");
        assert!(!query.full_text_search);
        assert_eq!(
            query.advanced_search_params(),
            vec![
                ("q".to_string(), "nasa".to_string()),
                ("fl[0]".to_string(), "title".to_string()),
                ("fl[1]".to_string(), "identifier".to_string()),
                ("sort[0]".to_string(), "downloads desc".to_string()),
                ("output".to_string(), "json".to_string()),
            ]
        );
    }

    #[test]
    fn builds_fts_payload() {
        let query = SearchQuery::new("nasa").with_full_text_search(false);
        let payload = query.fts_payload(Some(250), Some("mediatype"));
        assert_eq!(
            payload,
            vec![
                ("q".to_string(), "!L nasa".to_string()),
                ("size".to_string(), "250".to_string()),
                ("from".to_string(), "0".to_string()),
                ("scroll".to_string(), "false".to_string()),
                ("scope".to_string(), "mediatype".to_string()),
            ]
        );
    }
}
