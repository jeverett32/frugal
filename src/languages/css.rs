use std::path::Path;

use super::SkeletonOutput;

pub fn skeletonize(_path: &Path, source: &str) -> SkeletonOutput {
    let normalized = normalize_newlines(source);
    let stripped = strip_comments(&normalized);
    let body = summarize_rules(&stripped, 0);
    let body = if body.trim().is_empty() {
        "...\n".to_string()
    } else {
        body
    };

    SkeletonOutput {
        fence_label: "css",
        body,
        is_placeholder: false,
    }
}

fn summarize_rules(source: &str, indent: usize) -> String {
    let mut output = String::new();
    let bytes = source.as_bytes();
    let mut idx = 0;

    while idx < bytes.len() {
        skip_ws(source, &mut idx);
        if idx >= bytes.len() {
            break;
        }

        let header_start = idx;
        let mut quote = None;

        while idx < bytes.len() {
            let ch = bytes[idx];
            if let Some(active) = quote {
                if ch == active {
                    quote = None;
                } else if ch == b'\\' {
                    idx += 1;
                }
            } else if ch == b'"' || ch == b'\'' {
                quote = Some(ch);
            } else if ch == b'{' || ch == b';' {
                break;
            }
            idx += 1;
        }

        let header = collapse_whitespace(&source[header_start..idx]);
        if header.is_empty() {
            idx += 1;
            continue;
        }

        if idx >= bytes.len() || bytes[idx] == b';' {
            push_line(&mut output, indent, &header);
            idx += 1;
            continue;
        }

        let body_start = idx + 1;
        if let Some(body_end) = find_matching_brace(source, idx) {
            let inner = &source[body_start..body_end];
            render_block(&mut output, &header, inner, indent);
            idx = body_end + 1;
        } else {
            break;
        }
    }

    output
}

fn render_block(output: &mut String, header: &str, inner: &str, indent: usize) {
    if header.starts_with('@') {
        if block_contains_rules(inner) && !header.starts_with("@keyframes") {
            push_line(output, indent, &format!("{header} {{"));
            let nested = summarize_rules(inner, indent + 2);
            for line in nested.lines() {
                push_line(output, 0, line);
            }
            push_line(output, indent, "}");
        } else {
            push_line(output, indent, &header_without_body(header));
        }
        return;
    }

    push_line(output, indent, &collapse_selector_list(header));
}

fn block_contains_rules(source: &str) -> bool {
    let bytes = source.as_bytes();
    let mut idx = 0;
    let mut quote = None;

    while idx < bytes.len() {
        let ch = bytes[idx];
        if let Some(active) = quote {
            if ch == active {
                quote = None;
            } else if ch == b'\\' {
                idx += 1;
            }
        } else if ch == b'"' || ch == b'\'' {
            quote = Some(ch);
        } else if ch == b'{' {
            return true;
        }
        idx += 1;
    }

    false
}

fn find_matching_brace(source: &str, open_idx: usize) -> Option<usize> {
    let bytes = source.as_bytes();
    let mut idx = open_idx + 1;
    let mut depth = 1;
    let mut quote = None;

    while idx < bytes.len() {
        let ch = bytes[idx];
        if let Some(active) = quote {
            if ch == active {
                quote = None;
            } else if ch == b'\\' {
                idx += 1;
            }
        } else if ch == b'"' || ch == b'\'' {
            quote = Some(ch);
        } else if ch == b'{' {
            depth += 1;
        } else if ch == b'}' {
            depth -= 1;
            if depth == 0 {
                return Some(idx);
            }
        }
        idx += 1;
    }

    None
}

fn collapse_selector_list(header: &str) -> String {
    header
        .split(',')
        .map(collapse_whitespace)
        .collect::<Vec<_>>()
        .join(", ")
}

fn header_without_body(header: &str) -> String {
    collapse_whitespace(header)
}

fn strip_comments(source: &str) -> String {
    let bytes = source.as_bytes();
    let mut out = String::with_capacity(source.len());
    let mut idx = 0;
    let mut quote = None;

    while idx < bytes.len() {
        let ch = bytes[idx];
        if let Some(active) = quote {
            out.push(ch as char);
            if ch == active {
                quote = None;
            } else if ch == b'\\' && idx + 1 < bytes.len() {
                idx += 1;
                out.push(bytes[idx] as char);
            }
            idx += 1;
            continue;
        }

        if ch == b'"' || ch == b'\'' {
            quote = Some(ch);
            out.push(ch as char);
            idx += 1;
            continue;
        }

        if ch == b'/' && idx + 1 < bytes.len() && bytes[idx + 1] == b'*' {
            idx += 2;
            while idx + 1 < bytes.len() && !(bytes[idx] == b'*' && bytes[idx + 1] == b'/') {
                if bytes[idx] == b'\n' {
                    out.push('\n');
                }
                idx += 1;
            }
            idx += 2;
            continue;
        }

        out.push(ch as char);
        idx += 1;
    }

    out
}

fn collapse_whitespace(value: &str) -> String {
    value.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn skip_ws(source: &str, idx: &mut usize) {
    let bytes = source.as_bytes();
    while *idx < bytes.len() && bytes[*idx].is_ascii_whitespace() {
        *idx += 1;
    }
}

fn push_line(output: &mut String, indent: usize, line: &str) {
    output.push_str(&" ".repeat(indent));
    output.push_str(line.trim_end());
    output.push('\n');
}

fn normalize_newlines(source: &str) -> String {
    source.replace("\r\n", "\n").replace('\r', "\n")
}

#[cfg(test)]
mod tests {
    use super::skeletonize;
    use std::path::Path;

    #[test]
    fn extracts_compact_css_outline() {
        let source = r#"@import url("base.css");

.app-shell, .app-shell[data-state="ready"] {
  display: grid;
}

@media (min-width: 48rem) {
  .app-shell {
    grid-template-columns: 1fr 18rem;
  }

  nav.primary {
    position: sticky;
  }
}

@keyframes spin {
  from { transform: rotate(0deg); }
  to { transform: rotate(1turn); }
}
"#;

        let output = skeletonize(Path::new("web/app.css"), source);

        assert_eq!(output.fence_label, "css");
        assert!(!output.is_placeholder);
        assert_eq!(
            output.body,
            concat!(
                "@import url(\"base.css\")\n",
                ".app-shell, .app-shell[data-state=\"ready\"]\n",
                "@media (min-width: 48rem) {\n",
                "  .app-shell\n",
                "  nav.primary\n",
                "}\n",
                "@keyframes spin\n",
            )
        );
    }
}
