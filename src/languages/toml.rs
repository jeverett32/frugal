use std::path::Path;

use super::SkeletonOutput;

pub fn skeletonize(_path: &Path, source: &str) -> SkeletonOutput {
    let body = match source.parse::<::toml::Value>() {
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
        fence_label: "toml",
        body,
        is_placeholder: false,
    }
}

fn render_value(key: Option<&str>, value: &::toml::Value, indent: usize, out: &mut String) {
    let pad = " ".repeat(indent);
    match value {
        ::toml::Value::Table(table) => {
            if let Some(key) = key {
                out.push_str(&format!("{pad}{key}:\n"));
            }
            let mut entries = table.iter().collect::<Vec<_>>();
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
        ::toml::Value::Array(items) => {
            out.push_str(&format!(
                "{pad}{}: [len={}]\n",
                key.unwrap_or("<root>"),
                items.len()
            ));
        }
        ::toml::Value::String(_) => {
            out.push_str(&format!("{pad}{} = <string>\n", key.unwrap_or("<root>")))
        }
        ::toml::Value::Integer(_) => {
            out.push_str(&format!("{pad}{} = <int>\n", key.unwrap_or("<root>")))
        }
        ::toml::Value::Float(_) => {
            out.push_str(&format!("{pad}{} = <float>\n", key.unwrap_or("<root>")))
        }
        ::toml::Value::Boolean(_) => {
            out.push_str(&format!("{pad}{} = <bool>\n", key.unwrap_or("<root>")))
        }
        ::toml::Value::Datetime(_) => {
            out.push_str(&format!("{pad}{} = <datetime>\n", key.unwrap_or("<root>")))
        }
    }
}
