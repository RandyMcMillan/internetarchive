use std::collections::BTreeMap;

use serde_json::{Map, Value, json};

use crate::utils::{delete_items_from_dict, needs_quote};

fn is_indexed_key(key: &str) -> bool {
    key.contains('[') && key.contains(']')
}

fn get_base_key(key: &str) -> &str {
    key.split('[').next().unwrap_or(key)
}

fn get_index(key: &str) -> Option<usize> {
    let start = key.find('[')? + 1;
    let end = key.find(']')?;
    key[start..end].parse().ok()
}

/// Normalize and merge metadata before building a JSON patch.
pub fn prepare_metadata(
    metadata: &Map<String, Value>,
    source_metadata: Option<&Map<String, Value>>,
    append: bool,
    append_list: bool,
    insert: bool,
) -> Map<String, Value> {
    let source = source_metadata.cloned().unwrap_or_default();
    let mut metadata = metadata.clone();
    let mut prepared = Map::new();

    if insert && !metadata.keys().all(|key| is_indexed_key(key)) {
        let mut extra = Vec::new();
        for key in metadata.keys() {
            if !is_indexed_key(key) {
                extra.push((format!("{key}[0]"), metadata.get(key).cloned().unwrap()));
            }
        }
        for (key, value) in extra {
            metadata.insert(key, value);
        }
    }

    for (key, value) in metadata.iter() {
        if is_indexed_key(key) {
            continue;
        }
        if append_list {
            if let Some(existing) = source.get(key) {
                let mut existing_list = existing.as_array().cloned().unwrap_or_else(|| vec![existing.clone()]);
                if !existing_list.iter().any(|item| item == value) {
                    existing_list.push(value.clone());
                }
                prepared.insert(key.clone(), Value::Array(existing_list));
                continue;
            }
        }
        if append {
            if let Some(existing) = source.get(key) {
                if let Some(existing_str) = existing.as_str() {
                    prepared.insert(key.clone(), Value::String(format!("{existing_str} {value}")));
                    continue;
                }
            }
        }
        prepared.insert(key.clone(), value.clone());
    }

    let mut indexed_keys: BTreeMap<String, usize> = BTreeMap::new();
    let mut remove_indexes: BTreeMap<String, Vec<usize>> = BTreeMap::new();

    for key in metadata.keys() {
        if !is_indexed_key(key) {
            continue;
        }
        let base = get_base_key(key).to_string();
        let Some(idx) = get_index(key) else { continue };
        let source_list = source
            .get(&base)
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_else(|| {
                source
                    .get(&base)
                    .cloned()
                    .map(|value| vec![value])
                    .unwrap_or_default()
            });

        indexed_keys.entry(base.clone()).or_insert(source_list.len());
        let current_len = metadata.len();
        let entry = prepared.entry(base.clone()).or_insert_with(|| {
            let mut list = source_list;
            if current_len > list.len() {
                list.resize(current_len, Value::Null);
            }
            Value::Array(list)
        });

        let list = entry.as_array_mut().expect("prepared value must be array");
        while list.len() <= idx {
            list.push(Value::Null);
        }

        match metadata.get(key).unwrap() {
            Value::String(value) if value == "REMOVE_TAG" => {
                remove_indexes.entry(base).or_default().push(idx);
                list[idx] = Value::Null;
            }
            value if insert => {
                if let Some(pos) = list.iter().position(|existing| existing == value) {
                    list.remove(pos);
                }
                list.insert(idx, value.clone());
            }
            value => {
                list[idx] = value.clone();
            }
        }
    }

    for (base, mut indexes) in remove_indexes {
        if let Some(Value::Array(list)) = prepared.get_mut(&base) {
            indexes.sort_unstable_by(|a, b| b.cmp(a));
            for idx in indexes {
                if idx < list.len() {
                    list.remove(idx);
                }
            }
            list.retain(|value| !value.is_null());
        }
    }

    prepared.retain(|_, value| !value.is_null());
    prepared
}

/// Create a JSON Patch from metadata changes.
pub fn prepare_patch(
    metadata: &Value,
    source_metadata: &Value,
    append: bool,
    expect: Option<&Map<String, Value>>,
    append_list: bool,
    insert: bool,
) -> Vec<Value> {
    let mut destination = source_metadata.clone();
    let prepared_metadata = match metadata {
        Value::Array(array) => Value::Array(array.clone()),
        Value::Object(map) => Value::Object(prepare_metadata(
            map,
            source_metadata.as_object(),
            append,
            append_list,
            insert,
        )),
        _ => Value::Array(vec![metadata.clone()]),
    };

    if let Value::Object(dest_map) = &mut destination {
        if let Value::Object(prepared) = &prepared_metadata {
            for (key, value) in prepared {
                dest_map.insert(key.clone(), value.clone());
            }
        }
    } else if matches!(metadata, Value::Array(_)) {
        destination = prepared_metadata;
    } else if let Value::Array(array) = &prepared_metadata {
        destination = Value::Array(array.clone());
    }

    destination = delete_items_from_dict(destination, "REMOVE_TAG");
    let patch = json_patch::diff(source_metadata, &destination);
    let mut result: Vec<Value> = Vec::new();

    if let Some(expect) = expect {
        for (key, value) in expect {
            let path = if key.contains('[') {
                let base = get_base_key(key);
                let idx = get_index(key).unwrap_or(0);
                format!("/{base}/{idx}")
            } else {
                format!("/{key}")
            };
            result.push(json!({"op":"test","path":path,"value":value}));
        }
    }

    result.extend(patch.into_iter().map(|op| json!(op)));
    result
}

/// Create a JSON Patch for a specific metadata target path.
pub fn prepare_target_patch(
    metadata: &Value,
    source_metadata: &Value,
    append: bool,
    target: &str,
    append_list: bool,
    insert: bool,
    expect: Option<&Map<String, Value>>,
) -> Vec<Value> {
    let mut current = source_metadata;
    for part in target.split('/') {
        current = match current {
            Value::Array(list) => {
                let idx: usize = part.parse().unwrap_or(0);
                list.get(idx).unwrap_or(current)
            }
            Value::Object(map) => map.get(part).unwrap_or(current),
            _ => current,
        };
    }
    prepare_patch(metadata, current, append, expect, append_list, insert)
}

/// Create a JSON Patch for file-level metadata.
pub fn prepare_files_patch(
    metadata: &Value,
    files_metadata: &[Value],
    target: &str,
    append: bool,
    append_list: bool,
    insert: bool,
    expect: Option<&Map<String, Value>>,
) -> Vec<Value> {
    let filename = target.split('/').nth(1).unwrap_or("");
    for file_meta in files_metadata {
        if file_meta.get("name").and_then(Value::as_str) == Some(filename) {
            return prepare_patch(metadata, file_meta, append, expect, append_list, insert);
        }
    }
    Vec::new()
}

/// Add `x-archive-*` metadata headers to a header map.
pub fn prepare_metadata_headers(
    headers: &mut BTreeMap<String, String>,
    metadata: &Map<String, Value>,
    file_metadata: &Map<String, Value>,
    queue_derive: bool,
) {
    headers.insert(
        "x-archive-auto-make-bucket".to_string(),
        "1".to_string(),
    );
    headers.insert(
        "x-archive-queue-derive".to_string(),
        if queue_derive { "1" } else { "0" }.to_string(),
    );
    add_metadata_headers(headers, metadata, "meta");
    add_metadata_headers(headers, file_metadata, "filemeta");
}

fn add_metadata_headers(
    headers: &mut BTreeMap<String, String>,
    metadata: &Map<String, Value>,
    meta_type: &str,
) {
    for (key, values) in metadata {
        let values = match values {
            Value::Array(list) => list.clone(),
            other => vec![other.clone()],
        };
        for (idx, value) in values.into_iter().enumerate() {
            if value.is_null() {
                continue;
            }
            let header_key = format!("x-archive-{meta_type}{idx:02}-{key}").replace('_', "--");
            let header_value = value.as_str().map_or(value.to_string(), |s| {
                if needs_quote(s) {
                    format!("uri({})", urlencoding::encode(s))
                } else {
                    s.to_string()
                }
            });
            headers.insert(header_key, header_value);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prepares_basic_metadata() {
        let metadata = serde_json::json!({"title":"New Title"});
        let source = serde_json::json!({"title":"Old Title"});
        let patch = prepare_patch(&metadata, &source, false, None, false, false);
        assert!(!patch.is_empty());
    }

    #[test]
    fn builds_metadata_headers() {
        let mut headers = BTreeMap::new();
        let metadata = Map::from_iter([("source".to_string(), Value::String("a b".to_string()))]);
        prepare_metadata_headers(&mut headers, &metadata, &Map::new(), true);
        assert_eq!(headers.get("x-archive-auto-make-bucket"), Some(&"1".to_string()));
    }
}
