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
}
