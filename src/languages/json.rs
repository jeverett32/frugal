use std::path::Path;

use serde_json::Value;

use super::SkeletonOutput;

pub fn skeletonize(_path: &Path, source: &str) -> SkeletonOutput {
    let body = match serde_json::from_str::<Value>(source) {
        Ok(value) => {
            let mut out = String::new();
            render_value(None, &value, 0, &mut out);
            if out.is_empty() {
                "...\n".to_string()
            } else {
                out
            }
        }
        Err(_) => "...\n".to_string(),
    };

    SkeletonOutput {
        fence_label: "json",
        body,
        is_placeholder: false,
    }
}

fn render_value(key: Option<&str>, value: &Value, indent: usize, out: &mut String) {
    let pad = " ".repeat(indent);
    match value {
        Value::Object(map) => {
            if let Some(key) = key {
                out.push_str(&format!("{pad}{key}:\n"));
            }
            let mut entries = map.iter().collect::<Vec<_>>();
            entries.sort_by_key(|(child_key, _)| *child_key);
            for (child_key, child_value) in entries {
                render_value(
                    Some(child_key),
                    child_value,
                    indent + if key.is_some() { 2 } else { 0 },
                    out,
                );
            }
        }
        Value::Array(items) => {
            let label = key.unwrap_or("<root>");
            out.push_str(&format!("{pad}{label}: [len={}]\n", items.len()));
            if let Some(first) = items.first() {
                render_value(Some("-"), first, indent + 2, out);
            }
        }
        Value::String(_) => out.push_str(&format!("{pad}{}: <string>\n", key.unwrap_or("<root>"))),
        Value::Number(_) => out.push_str(&format!("{pad}{}: <number>\n", key.unwrap_or("<root>"))),
        Value::Bool(_) => out.push_str(&format!("{pad}{}: <bool>\n", key.unwrap_or("<root>"))),
        Value::Null => out.push_str(&format!("{pad}{}: null\n", key.unwrap_or("<root>"))),
    }
}
