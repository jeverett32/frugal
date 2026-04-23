use std::path::Path;

use super::SkeletonOutput;

pub fn skeletonize(_path: &Path, source: &str) -> SkeletonOutput {
    let normalized = source.replace("\r\n", "\n").replace('\r', "\n");
    let mut out = Vec::new();

    for raw in normalized.lines() {
        let line = raw.trim_end();
        let trimmed = line.trim_start();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let indent = line.len() - trimmed.len();
        let prefix = " ".repeat(indent);
        if let Some(rest) = trimmed.strip_prefix("- ") {
            if !rest.starts_with('"') && !rest.starts_with('\'') {
                if let Some((key, value)) = rest.split_once(':') {
                    let value = if value.trim().is_empty() { "" } else { " ..." };
                    out.push(format!("{prefix}- {}:{value}", key.trim()));
                } else {
                    out.push(format!("{prefix}- ..."));
                }
            } else {
                out.push(format!("{prefix}- ..."));
            }
        } else if let Some((key, value)) = trimmed.split_once(':') {
            let value = if value.trim().is_empty() { "" } else { " ..." };
            out.push(format!("{prefix}{}:{value}", key.trim()));
        }
    }

    let body = if out.is_empty() {
        "...\n".to_string()
    } else {
        format!("{}\n", out.join("\n"))
    };

    SkeletonOutput {
        fence_label: "yaml",
        body,
        is_placeholder: false,
    }
}
