use std::path::Path;

use super::SkeletonOutput;

pub fn skeletonize(_path: &Path, source: &str) -> SkeletonOutput {
    let normalized = source.replace("\r\n", "\n").replace('\r', "\n");
    let mut out = Vec::new();

    for line in normalized.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if trimmed == "{" || trimmed == "}" {
            continue;
        }
        if trimmed.starts_with("#!") {
            out.push(trimmed.to_string());
            continue;
        }
        if trimmed.starts_with("export ") {
            let key = trimmed
                .trim_start_matches("export ")
                .split('=')
                .next()
                .unwrap_or(trimmed);
            out.push(format!("export {}=...", key.trim()));
            continue;
        }
        if trimmed.starts_with(". ") || trimmed.starts_with("source ") {
            out.push(trimmed.to_string());
            continue;
        }
        if trimmed.ends_with("() {") || trimmed.ends_with("(){") {
            out.push(trimmed.to_string());
            continue;
        }
        if let Some((name, rest)) = trimmed.split_once('(') {
            if rest.trim_start().starts_with(')') && trimmed.contains('{') {
                out.push(format!("{}() {{", name.trim()));
                continue;
            }
        }
        let cmd = trimmed.split_whitespace().next().unwrap_or(trimmed);
        if !cmd.starts_with('#') {
            out.push(format!("{cmd} ..."));
        }
    }

    out.dedup();
    let body = if out.is_empty() {
        "...\n".to_string()
    } else {
        format!("{}\n", out.join("\n"))
    };

    SkeletonOutput {
        fence_label: "bash",
        body,
        is_placeholder: false,
    }
}
