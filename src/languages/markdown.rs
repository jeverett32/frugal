use std::path::Path;

use super::SkeletonOutput;

pub fn skeletonize(_path: &Path, source: &str) -> SkeletonOutput {
    let normalized = source.replace("\r\n", "\n").replace('\r', "\n");
    let mut out = Vec::new();
    let mut bullets = 0usize;
    let mut tasks = 0usize;

    for line in normalized.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('#') {
            out.push(trimmed.to_string());
        } else if trimmed.starts_with("- [ ]")
            || trimmed.starts_with("- [x]")
            || trimmed.starts_with("* [ ]")
            || trimmed.starts_with("* [x]")
        {
            tasks += 1;
        } else if trimmed.starts_with("- ") || trimmed.starts_with("* ") {
            bullets += 1;
        } else if let Some(lang) = trimmed.strip_prefix("```") {
            let lang = lang.trim();
            if !lang.is_empty() {
                out.push(format!("```{lang}"));
            }
        }
    }

    if bullets > 0 {
        out.push(format!("bullets: {bullets}"));
    }
    if tasks > 0 {
        out.push(format!("tasks: {tasks}"));
    }

    let body = if out.is_empty() {
        "...\n".to_string()
    } else {
        format!("{}\n", out.join("\n"))
    };

    SkeletonOutput {
        fence_label: "markdown",
        body,
        is_placeholder: false,
    }
}
