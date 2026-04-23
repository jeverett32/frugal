use std::path::Path;

use super::SkeletonOutput;

pub fn skeletonize(_path: &Path, source: &str) -> SkeletonOutput {
    let normalized = normalize_newlines(source);
    let summary = summarize_html(&normalized);
    let body = if summary.is_empty() {
        "...\n".to_string()
    } else {
        summary
    };

    SkeletonOutput {
        fence_label: "html",
        body,
        is_placeholder: false,
    }
}

fn summarize_html(source: &str) -> String {
    let mut collector = Collector::default();
    let mut stack = Vec::new();
    let bytes = source.as_bytes();
    let mut idx = 0;

    while idx < bytes.len() {
        if bytes[idx] != b'<' {
            idx += 1;
            continue;
        }

        if starts_with(source, idx, "<!--") {
            if let Some(end) = source[idx + 4..].find("-->") {
                idx += 4 + end + 3;
            } else {
                break;
            }
            continue;
        }

        if starts_with(source, idx, "</") {
            if let Some((name, end)) = parse_closing_tag(source, idx) {
                close_tag(&mut collector, &mut stack, &name, source, end);
                idx = end;
                continue;
            }
        }

        if starts_with(source, idx, "<!") || starts_with(source, idx, "<?") {
            if let Some(end) = find_tag_end(source, idx + 1) {
                idx = end;
            } else {
                break;
            }
            continue;
        }

        if let Some((tag, end)) = parse_open_tag(source, idx) {
            handle_open_tag(&mut collector, &mut stack, &tag, end);
            idx = end;
            continue;
        }

        idx += 1;
    }

    collector.finish()
}

#[derive(Default)]
struct Collector {
    title: Option<String>,
    headings: Vec<String>,
    includes: Vec<String>,
    structure: Vec<String>,
}

impl Collector {
    fn finish(self) -> String {
        let mut output = String::new();

        if let Some(title) = self.title {
            push_line(&mut output, 0, &format!("title: {title}"));
        }

        if !self.headings.is_empty() {
            push_line(
                &mut output,
                0,
                &format!("headings: {}", self.headings.join("; ")),
            );
        }

        if !self.includes.is_empty() {
            push_line(
                &mut output,
                0,
                &format!("includes: {}", self.includes.join("; ")),
            );
        }

        if !self.structure.is_empty() {
            push_line(&mut output, 0, "structure:");
            for entry in self.structure {
                output.push_str(&entry);
                output.push('\n');
            }
        }

        output
    }
}

#[derive(Clone)]
struct OpenTag {
    name: String,
    attrs: Vec<(String, String)>,
    self_closing: bool,
}

struct StackEntry {
    name: String,
    text_capture: Option<TextCapture>,
}

struct TextCapture {
    kind: CaptureKind,
    start: usize,
}

enum CaptureKind {
    Title,
    Heading(String),
}

fn handle_open_tag(
    collector: &mut Collector,
    stack: &mut Vec<StackEntry>,
    tag: &OpenTag,
    tag_end: usize,
) {
    let name = tag.name.as_str();

    if let Some(include) = summarize_include(tag) {
        collector.includes.push(include);
    }

    let relevant_depth = stack
        .iter()
        .filter(|entry| is_relevant_structure(&entry.name))
        .count();

    if let Some(line) = summarize_structure(tag, relevant_depth) {
        collector.structure.push(line);
    }

    let capture = if name == "title" {
        Some(TextCapture {
            kind: CaptureKind::Title,
            start: tag_end,
        })
    } else if is_heading(name) {
        Some(TextCapture {
            kind: CaptureKind::Heading(name.to_string()),
            start: tag_end,
        })
    } else {
        None
    };

    if !tag.self_closing && !is_void_tag(name) {
        stack.push(StackEntry {
            name: name.to_string(),
            text_capture: capture,
        });
    }
}

fn close_tag(
    collector: &mut Collector,
    stack: &mut Vec<StackEntry>,
    name: &str,
    source: &str,
    close_start: usize,
) {
    let Some(position) = stack.iter().rposition(|entry| entry.name == name) else {
        return;
    };

    let mut drained = stack.split_off(position);
    if let Some(entry) = drained.first_mut() {
        if let Some(capture) = entry.text_capture.take() {
            let text = extract_text(source, capture.start, close_start);
            let text = clip_text(&collapse_whitespace(&text), 80);
            if !text.is_empty() {
                match capture.kind {
                    CaptureKind::Title => collector.title = Some(text),
                    CaptureKind::Heading(level) => {
                        collector.headings.push(format!("{level} {text}"));
                    }
                }
            }
        }
    }

    drained.remove(0);
    for entry in drained {
        stack.push(entry);
    }
}

fn summarize_include(tag: &OpenTag) -> Option<String> {
    match tag.name.as_str() {
        "script" => {
            let src = attr_value(tag, "src");
            let typ = attr_value(tag, "type");
            let defer = has_attr(tag, "defer");
            let async_attr = has_attr(tag, "async");

            let mut line = if let Some(src) = src {
                format!("script:{src}")
            } else {
                "script:inline".to_string()
            };

            if let Some(typ) = typ.filter(|typ| !typ.eq_ignore_ascii_case("text/javascript")) {
                line.push_str(&format!(" [{typ}]"));
            }
            if defer {
                line.push_str(" defer");
            }
            if async_attr {
                line.push_str(" async");
            }
            Some(line)
        }
        "style" => Some("style:inline".to_string()),
        "link" => {
            let rel = attr_value(tag, "rel")?;
            if !rel_contains(&rel, "stylesheet") && !rel_contains(&rel, "preload") {
                return None;
            }
            let href = attr_value(tag, "href").unwrap_or_else(|| "...".to_string());
            let as_attr = attr_value(tag, "as");
            let mut line = format!("link:{href}");
            if rel_contains(&rel, "stylesheet") {
                line.push_str(" [stylesheet]");
            } else {
                line.push_str(" [preload]");
            }
            if let Some(as_attr) = as_attr {
                line.push_str(&format!(" as={as_attr}"));
            }
            Some(line)
        }
        _ => None,
    }
}

fn summarize_structure(tag: &OpenTag, relevant_depth: usize) -> Option<String> {
    let name = tag.name.as_str();
    let include = if is_relevant_structure(name) {
        true
    } else {
        relevant_depth == 0
            && (attr_value(tag, "id").is_some() || attr_value(tag, "class").is_some())
    };

    if !include || relevant_depth > 2 {
        return None;
    }

    let mut line = String::new();
    line.push_str(&"  ".repeat(relevant_depth));
    line.push_str("- ");
    line.push_str(name);

    if let Some(id) = attr_value(tag, "id").filter(|id| !id.is_empty()) {
        line.push('#');
        line.push_str(&sanitize_token(&id));
    }

    if let Some(classes) = attr_value(tag, "class") {
        for class_name in classes.split_whitespace().take(3) {
            let token = sanitize_token(class_name);
            if !token.is_empty() {
                line.push('.');
                line.push_str(&token);
            }
        }
    }

    if name == "form" {
        let mut extras = Vec::new();
        if let Some(action) = attr_value(tag, "action").filter(|value| !value.is_empty()) {
            extras.push(format!("action={action}"));
        }
        if let Some(method) = attr_value(tag, "method").filter(|value| !value.is_empty()) {
            extras.push(format!("method={}", method.to_lowercase()));
        }
        if !extras.is_empty() {
            line.push(' ');
            line.push('[');
            line.push_str(&extras.join(" "));
            line.push(']');
        }
    }

    Some(line)
}

fn parse_open_tag(source: &str, start: usize) -> Option<(OpenTag, usize)> {
    let end = find_tag_end(source, start + 1)?;
    let inner = &source[start + 1..end - 1];
    let trimmed = inner.trim();
    if trimmed.is_empty() || trimmed.starts_with('/') {
        return None;
    }

    let self_closing = trimmed.ends_with('/');
    let mut chars = trimmed.chars().peekable();
    let mut name = String::new();
    while let Some(ch) = chars.peek().copied() {
        if ch.is_ascii_alphanumeric() || ch == '-' || ch == ':' {
            name.push(ch.to_ascii_lowercase());
            chars.next();
        } else {
            break;
        }
    }

    if name.is_empty() {
        return None;
    }

    let attrs_src = &trimmed[name.len()..];
    let attrs = parse_attributes(attrs_src);

    Some((
        OpenTag {
            name,
            attrs,
            self_closing,
        },
        end,
    ))
}

fn parse_closing_tag(source: &str, start: usize) -> Option<(String, usize)> {
    let end = find_tag_end(source, start + 2)?;
    let inner = &source[start + 2..end - 1];
    let name = inner
        .trim()
        .chars()
        .take_while(|ch| ch.is_ascii_alphanumeric() || *ch == '-' || *ch == ':')
        .collect::<String>()
        .to_ascii_lowercase();
    if name.is_empty() {
        None
    } else {
        Some((name, end))
    }
}

fn parse_attributes(source: &str) -> Vec<(String, String)> {
    let bytes = source.as_bytes();
    let mut attrs = Vec::new();
    let mut idx = 0;

    while idx < bytes.len() {
        while idx < bytes.len() && bytes[idx].is_ascii_whitespace() {
            idx += 1;
        }
        if idx >= bytes.len() || bytes[idx] == b'/' {
            break;
        }

        let name_start = idx;
        while idx < bytes.len()
            && (bytes[idx].is_ascii_alphanumeric()
                || bytes[idx] == b'-'
                || bytes[idx] == b':'
                || bytes[idx] == b'_')
        {
            idx += 1;
        }
        if idx == name_start {
            idx += 1;
            continue;
        }

        let name = source[name_start..idx].to_ascii_lowercase();
        while idx < bytes.len() && bytes[idx].is_ascii_whitespace() {
            idx += 1;
        }

        let value = if idx < bytes.len() && bytes[idx] == b'=' {
            idx += 1;
            while idx < bytes.len() && bytes[idx].is_ascii_whitespace() {
                idx += 1;
            }
            parse_attr_value(source, &mut idx)
        } else {
            String::new()
        };

        attrs.push((name, collapse_whitespace(&value)));
    }

    attrs
}

fn parse_attr_value(source: &str, idx: &mut usize) -> String {
    let bytes = source.as_bytes();
    if *idx >= bytes.len() {
        return String::new();
    }

    let quote = bytes[*idx];
    if quote == b'"' || quote == b'\'' {
        *idx += 1;
        let start = *idx;
        while *idx < bytes.len() && bytes[*idx] != quote {
            *idx += 1;
        }
        let value = source[start..(*idx).min(bytes.len())].to_string();
        if *idx < bytes.len() {
            *idx += 1;
        }
        value
    } else {
        let start = *idx;
        while *idx < bytes.len() && !bytes[*idx].is_ascii_whitespace() && bytes[*idx] != b'/' {
            *idx += 1;
        }
        source[start..*idx].to_string()
    }
}

fn attr_value(tag: &OpenTag, name: &str) -> Option<String> {
    tag.attrs
        .iter()
        .find(|(attr, _)| attr == name)
        .map(|(_, value)| clip_text(value.trim(), 60))
}

fn has_attr(tag: &OpenTag, name: &str) -> bool {
    tag.attrs.iter().any(|(attr, _)| attr == name)
}

fn extract_text(source: &str, mut start: usize, end: usize) -> String {
    if start == 0 {
        start = 0;
    }
    let slice = &source[start..end];
    let mut output = String::new();
    let mut in_tag = false;
    for ch in slice.chars() {
        match ch {
            '<' => in_tag = true,
            '>' if in_tag => {
                in_tag = false;
                output.push(' ');
            }
            _ if !in_tag => output.push(ch),
            _ => {}
        }
    }

    output
}

fn find_tag_end(source: &str, mut idx: usize) -> Option<usize> {
    let bytes = source.as_bytes();
    let mut quote = None;

    while idx < bytes.len() {
        let ch = bytes[idx];
        if let Some(active) = quote {
            if ch == active {
                quote = None;
            }
        } else if ch == b'"' || ch == b'\'' {
            quote = Some(ch);
        } else if ch == b'>' {
            return Some(idx + 1);
        }
        idx += 1;
    }

    None
}

fn is_relevant_structure(name: &str) -> bool {
    matches!(
        name,
        "header" | "footer" | "nav" | "main" | "section" | "article" | "aside" | "form"
    ) || name.contains('-')
}

fn is_heading(name: &str) -> bool {
    matches!(name, "h1" | "h2" | "h3" | "h4" | "h5" | "h6")
}

fn is_void_tag(name: &str) -> bool {
    matches!(
        name,
        "area"
            | "base"
            | "br"
            | "col"
            | "embed"
            | "hr"
            | "img"
            | "input"
            | "link"
            | "meta"
            | "param"
            | "source"
            | "track"
            | "wbr"
    )
}

fn rel_contains(value: &str, needle: &str) -> bool {
    value
        .split_whitespace()
        .any(|part| part.eq_ignore_ascii_case(needle))
}

fn sanitize_token(value: &str) -> String {
    value
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | ':'))
        .collect()
}

fn collapse_whitespace(value: &str) -> String {
    value.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn clip_text(value: &str, limit: usize) -> String {
    if value.chars().count() <= limit {
        return value.to_string();
    }

    let clipped = value
        .chars()
        .take(limit.saturating_sub(3))
        .collect::<String>();
    format!("{clipped}...")
}

fn push_line(output: &mut String, indent: usize, line: &str) {
    output.push_str(&" ".repeat(indent));
    output.push_str(line.trim_end());
    output.push('\n');
}

fn normalize_newlines(source: &str) -> String {
    source.replace("\r\n", "\n").replace('\r', "\n")
}

fn starts_with(source: &str, idx: usize, needle: &str) -> bool {
    source
        .as_bytes()
        .get(idx..idx + needle.len())
        .map(|slice| slice == needle.as_bytes())
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::skeletonize;
    use std::path::Path;

    #[test]
    fn extracts_compact_html_outline() {
        let source = r#"<!doctype html>
<html>
  <head>
    <title>Example Dashboard</title>
    <link rel="stylesheet" href="/app.css">
    <script src="/app.js" defer></script>
  </head>
  <body>
    <main id="app" class="shell">
      <h1>Dashboard</h1>
      <section class="hero">
        <user-card id="account"></user-card>
      </section>
    </main>
  </body>
</html>
"#;

        let output = skeletonize(Path::new("web/index.html"), source);

        assert_eq!(output.fence_label, "html");
        assert!(!output.is_placeholder);
        assert_eq!(
            output.body,
            concat!(
                "title: Example Dashboard\n",
                "headings: h1 Dashboard\n",
                "includes: link:/app.css [stylesheet]; script:/app.js defer\n",
                "structure:\n",
                "- main#app.shell\n",
                "  - section.hero\n",
                "    - user-card#account\n",
            )
        );
    }
}
