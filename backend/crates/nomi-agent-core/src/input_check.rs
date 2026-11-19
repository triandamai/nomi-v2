//! Checks on what a person edited in an approval card before the tool runs on it, and what a
//! tool handed back that the card can link to.

use serde_json::Value;

/// Keys a tool's output may carry its result's address under, best first.
const RESULT_URL_KEYS: &[&str] = &["result_url", "web_url", "permalink", "html_url", "url", "link"];

/// Why `input` can't be what `schema` describes, in words fit to show the person; `Ok` when it
/// fits. Covers the parts of JSON Schema the tools' own schemas use: `type`, `properties`,
/// `required`, `enum` and `additionalProperties: false` (nested objects and array items too).
pub fn check_input(schema: &Value, input: &Value) -> Result<(), String> {
    check_at(schema, input, "input")
}

fn type_matches(expected: &str, value: &Value) -> bool {
    match expected {
        "object" => value.is_object(),
        "array" => value.is_array(),
        "string" => value.is_string(),
        "boolean" => value.is_boolean(),
        "integer" => value.is_i64() || value.is_u64(),
        "number" => value.is_number(),
        "null" => value.is_null(),
        _ => true,
    }
}

fn check_at(schema: &Value, value: &Value, at: &str) -> Result<(), String> {
    if let Some(expected) = schema.get("type") {
        let ok = match expected {
            Value::String(t) => type_matches(t, value),
            Value::Array(ts) => ts.iter().filter_map(Value::as_str).any(|t| type_matches(t, value)),
            _ => true,
        };
        if !ok {
            return Err(format!("{at} must be of type {expected}"));
        }
    }
    if let Some(allowed) = schema.get("enum").and_then(Value::as_array) {
        if !allowed.contains(value) {
            return Err(format!("{at} must be one of the listed values"));
        }
    }
    if let (Some(object), Some(properties)) = (value.as_object(), schema.get("properties").and_then(Value::as_object)) {
        for name in schema.get("required").and_then(Value::as_array).into_iter().flatten().filter_map(Value::as_str) {
            if !object.contains_key(name) {
                return Err(format!("{at}.{name} is required"));
            }
        }
        for (key, child) in object {
            match properties.get(key) {
                Some(child_schema) => check_at(child_schema, child, &format!("{at}.{key}"))?,
                None if schema.get("additionalProperties") == Some(&Value::Bool(false)) => {
                    return Err(format!("{at}.{key} isn't something this tool takes"));
                }
                None => {}
            }
        }
    }
    if let (Some(items), Some(item_schema)) = (value.as_array(), schema.get("items")) {
        for (i, item) in items.iter().enumerate() {
            check_at(item_schema, item, &format!("{at}[{i}]"))?;
        }
    }
    Ok(())
}

/// The address a tool's output links to (the sent mail, the created page), when its output is
/// JSON carrying one as an http(s) URL, at the top or one object down.
pub fn result_url_in(output: &str) -> Option<String> {
    let parsed: Value = serde_json::from_str(output.trim()).ok()?;
    let find = |object: &Value| {
        RESULT_URL_KEYS.iter().find_map(|key| {
            let url = object.get(*key)?.as_str()?.trim();
            (url.starts_with("https://") || url.starts_with("http://")).then(|| url.to_string())
        })
    };
    find(&parsed).or_else(|| parsed.as_object()?.values().filter(|v| v.is_object()).find_map(find))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn send_schema() -> Value {
        json!({"type": "object", "required": ["to", "subject"], "additionalProperties": false,
               "properties": {"to": {"type": "string"}, "subject": {"type": "string"}, "urgent": {"type": "boolean"},
                              "tone": {"type": "string", "enum": ["formal", "casual"]}}})
    }

    #[test]
    fn edited_input_that_fits_the_schema_passes() {
        assert!(check_input(&send_schema(), &json!({"to": "a@b.co", "subject": "Hi", "tone": "formal"})).is_ok());
    }

    #[test]
    fn a_missing_required_field_is_named() {
        let err = check_input(&send_schema(), &json!({"to": "a@b.co"})).unwrap_err();
        assert!(err.contains("subject"), "{err}");
    }

    #[test]
    fn a_wrong_type_an_unknown_key_and_a_bad_enum_are_refused() {
        assert!(check_input(&send_schema(), &json!({"to": 5, "subject": "x"})).is_err());
        assert!(check_input(&send_schema(), &json!({"to": "a", "subject": "x", "cc": "z"})).is_err());
        assert!(check_input(&send_schema(), &json!({"to": "a", "subject": "x", "tone": "angry"})).is_err());
        assert!(check_input(&send_schema(), &json!("just text")).is_err());
    }

    #[test]
    fn nested_arrays_are_checked() {
        let schema = json!({"type": "object", "properties": {"tags": {"type": "array", "items": {"type": "string"}}}});
        assert!(check_input(&schema, &json!({"tags": ["a", "b"]})).is_ok());
        assert!(check_input(&schema, &json!({"tags": ["a", 2]})).is_err());
    }

    #[test]
    fn a_result_url_is_found_at_the_top_or_one_object_down() {
        assert_eq!(result_url_in(r#"{"id":"1","url":"https://mail.example.com/m/1"}"#).as_deref(), Some("https://mail.example.com/m/1"));
        assert_eq!(result_url_in(r#"{"data":{"permalink":"http://x.test/p"}}"#).as_deref(), Some("http://x.test/p"));
        assert_eq!(result_url_in(r#"{"result_url":"https://a.test/r","url":"https://a.test/other"}"#).as_deref(), Some("https://a.test/r"));
    }

    #[test]
    fn no_result_url_when_there_is_none_or_it_is_not_http() {
        assert_eq!(result_url_in("Sent it."), None);
        assert_eq!(result_url_in(r#"{"url":"javascript:alert(1)"}"#), None);
        assert_eq!(result_url_in(r#"{"id":"1"}"#), None);
    }
}
