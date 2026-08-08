use chrono::{Datelike, NaiveDateTime, Utc};
use serde_json::{Map, Value};

/// Statuses that mean a task is still active.
pub const ACTIVE_TASK_STATUSES: &[&str] = &["running", "queued", "paused"];

/// A lightweight Archive.org catalog task model.
#[derive(Debug, Clone, PartialEq)]
pub struct CatalogTask {
    pub data: Map<String, Value>,
}

impl CatalogTask {
    /// Construct a task from Archive.org JSON.
    pub fn from_json(value: &Value) -> Option<Self> {
        value.as_object().cloned().map(|data| Self { data })
    }

    /// Return the task identifier.
    pub fn task_id(&self) -> Option<u64> {
        self.data.get("task_id").and_then(Value::as_u64)
    }

    /// Return the task category.
    pub fn category(&self) -> Option<&str> {
        self.data.get("category").and_then(Value::as_str)
    }

    /// Return the task status.
    pub fn status(&self) -> Option<&str> {
        self.data.get("status").and_then(Value::as_str)
    }

    /// Return the task submission time.
    pub fn submittime(&self) -> Option<&str> {
        self.data.get("submittime").and_then(Value::as_str)
    }

    /// Return true if the task is still active.
    pub fn is_active(&self) -> bool {
        self.status()
            .map(|status| ACTIVE_TASK_STATUSES.contains(&status))
            .unwrap_or(false)
    }

    /// Select log lines using the same semantics as the Python helper.
    pub fn select_log_lines(text: &str, lines: Option<i64>) -> String {
        match lines {
            None => text.to_string(),
            Some(n) if n == 0 => String::new(),
            Some(n) if n < 0 => {
                let mut split: Vec<&str> = text.lines().collect();
                let keep = n.unsigned_abs() as usize;
                if keep >= split.len() {
                    return text.to_string();
                }
                split.drain(0..split.len() - keep);
                if text.ends_with('\n') {
                    let mut joined = split.join("\n");
                    joined.push('\n');
                    joined
                } else {
                    split.join("\n")
                }
            }
            Some(_) => String::new(),
        }
    }

    /// Sort tasks by submission date, newest last.
    pub fn sort_key(&self) -> chrono::DateTime<Utc> {
        let submittime = self.submittime().unwrap_or("1970-01-01 00:00:00");
        if let Ok(parsed) = NaiveDateTime::parse_from_str(submittime, "%Y-%m-%d %H:%M:%S.%f") {
            chrono::DateTime::<Utc>::from_naive_utc_and_offset(parsed, Utc)
        } else if let Ok(parsed) = NaiveDateTime::parse_from_str(submittime, "%Y-%m-%d %H:%M:%S") {
            chrono::DateTime::<Utc>::from_naive_utc_and_offset(parsed, Utc)
        } else {
            Utc::now()
        }
    }
}

/// Return true when a task status is active.
pub fn task_is_active(status: Option<&str>) -> bool {
    status
        .map(|status| ACTIVE_TASK_STATUSES.contains(&status))
        .unwrap_or(false)
}

/// Determine whether a task log should be followed based on a status.
pub fn task_log_should_continue(status: Option<&str>) -> bool {
    task_is_active(status)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_task_json() {
        let task = CatalogTask::from_json(&serde_json::json!({
            "task_id": 123,
            "category": "catalog",
            "status": "running",
            "submittime": "2026-05-28 12:00:00"
        }))
        .unwrap();
        assert_eq!(task.task_id(), Some(123));
        assert!(task.is_active());
    }

    #[test]
    fn selects_log_lines() {
        assert_eq!(
            CatalogTask::select_log_lines("a\nb\nc\nd\n", Some(-2)),
            "c\nd\n"
        );
        assert_eq!(CatalogTask::select_log_lines("a\nb\n", Some(0)), "");
    }

    #[test]
    fn sorts_by_date() {
        let task = CatalogTask::from_json(&serde_json::json!({
            "task_id": 123,
            "submittime": "2026-05-28 12:00:00"
        }))
        .unwrap();
        assert_eq!(task.sort_key().year(), 2026);
    }
}
