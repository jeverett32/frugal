use std::path::Path;

use super::SkeletonOutput;
use tree_sitter::{Node, Parser};

pub fn skeletonize(_path: &Path, source: &str) -> SkeletonOutput {
    let normalized = normalize_newlines(source);
    let mut parser = Parser::new();
    parser
        .set_language(&tree_sitter_python::LANGUAGE.into())
        .expect("python grammar should load");

    let Some(tree) = parser.parse(&normalized, None) else {
        return parse_failure();
    };

    let mut output = String::new();
    render_module(tree.root_node(), &normalized, &mut output);

    if output.is_empty() {
        output.push_str("...\n");
    }

    SkeletonOutput {
        fence_label: "python",
        body: output,
        is_placeholder: false,
    }
}

fn render_module(module: Node<'_>, source: &str, output: &mut String) {
    let mut cursor = module.walk();
    let children = module
        .named_children(&mut cursor)
        .collect::<Vec<_>>();
    let mut items = Vec::new();

    if let Some(docstring) = leading_docstring(&children, source) {
        items.push((0, docstring));
    }

    for child in children {
        match child.kind() {
            "function_definition" => items.push((child.start_position().row, render_function(child, source, 0))),
            "class_definition" => items.push((child.start_position().row, render_class(child, source, 0))),
            "decorated_definition" => items.push((child.start_position().row, render_decorated(child, source, 0))),
            _ => {}
        }
    }

    items.extend(top_level_constant_items(source));
    items.sort_by_key(|(row, _)| *row);

    for (_, block) in items {
        push_block(output, &block);
    }
}

fn render_decorated(node: Node<'_>, source: &str, indent: usize) -> String {
    let mut rendered = String::new();
    let definition = node
        .child_by_field_name("definition")
        .expect("decorated_definition should have definition");

    let decorators_src = slice(source, node.start_byte(), definition.start_byte());
    for line in decorators_src.lines() {
        let trimmed = line.trim();
        if !trimmed.is_empty() {
            push_line(&mut rendered, indent, trimmed);
        }
    }

    match definition.kind() {
        "function_definition" => rendered.push_str(&render_function(definition, source, indent)),
        "class_definition" => rendered.push_str(&render_class(definition, source, indent)),
        _ => {}
    }

    rendered
}

fn render_function(node: Node<'_>, source: &str, indent: usize) -> String {
    let mut rendered = String::new();
    let header = header_with_ellipsis(node, source);
    push_line(&mut rendered, indent, &header);

    if let Some(docstring) = immediate_docstring(node, source) {
        push_line(&mut rendered, indent + 4, &docstring);
    }

    rendered
}

fn render_class(node: Node<'_>, source: &str, indent: usize) -> String {
    let mut rendered = String::new();
    let header = class_header(node, source);
    push_line(&mut rendered, indent, &header);

    let mut has_body = false;
    if let Some(docstring) = immediate_docstring(node, source) {
        push_line(&mut rendered, indent + 4, &docstring);
        has_body = true;
    }

    if let Some(body) = node.child_by_field_name("body") {
        let mut cursor = body.walk();
        for child in body.named_children(&mut cursor) {
            match child.kind() {
                "function_definition" => {
                    push_block(&mut rendered, &render_function(child, source, indent + 4));
                    has_body = true;
                }
                "class_definition" => {
                    push_block(&mut rendered, &render_class(child, source, indent + 4));
                    has_body = true;
                }
                "decorated_definition" => {
                    push_block(&mut rendered, &render_decorated(child, source, indent + 4));
                    has_body = true;
                }
                _ => {}
            }
        }
    }

    if !has_body {
        push_line(&mut rendered, indent + 4, "...");
    }

    rendered
}

fn leading_docstring(children: &[Node<'_>], source: &str) -> Option<String> {
    children.first().and_then(|node| expression_docstring(*node, source))
}

fn immediate_docstring(node: Node<'_>, source: &str) -> Option<String> {
    let body = node.child_by_field_name("body")?;
    let mut cursor = body.walk();
    let first = body.named_children(&mut cursor).next()?;
    expression_docstring(first, source)
}

fn expression_docstring(node: Node<'_>, source: &str) -> Option<String> {
    if node.kind() != "expression_statement" {
        return None;
    }

    let mut cursor = node.walk();
    let expr = node.named_children(&mut cursor).next()?;
    let raw = match expr.kind() {
        "string" | "concatenated_string" => expr.utf8_text(source.as_bytes()).ok()?,
        _ => return None,
    };

    Some(docstring_literal(raw))
}

fn docstring_literal(raw: &str) -> String {
    let mut content = raw.trim().to_string();
    for quote in ["\"\"\"", "'''", "\"", "'"] {
        if content.starts_with(quote) && content.ends_with(quote) && content.len() >= quote.len() * 2
        {
            content = content[quote.len()..content.len() - quote.len()].to_string();
            break;
        }
    }

    let summary = content
        .lines()
        .map(str::trim)
        .find(|line| !line.is_empty())
        .unwrap_or("");

    format!("\"\"\"{}\"\"\"", summary.replace('\"', "\\\""))
}

fn header_with_ellipsis(node: Node<'_>, source: &str) -> String {
    let body = node
        .child_by_field_name("body")
        .expect("function/class should have body");
    let mut header = slice(source, node.start_byte(), body.start_byte()).trim_end().to_string();
    if header.ends_with(':') {
        header.push_str(" ...");
    } else {
        header.push_str(": ...");
    }
    header
}

fn class_header(node: Node<'_>, source: &str) -> String {
    let body = node
        .child_by_field_name("body")
        .expect("class should have body");
    slice(source, node.start_byte(), body.start_byte())
        .trim_end()
        .to_string()
}

fn push_block(output: &mut String, block: &str) {
    if block.trim().is_empty() {
        return;
    }
    if !output.is_empty() && !output.ends_with('\n') {
        output.push('\n');
    }
    if !output.is_empty() {
        output.push('\n');
    }
    output.push_str(block.trim_end());
    output.push('\n');
}

fn push_line(output: &mut String, indent: usize, line: &str) {
    output.push_str(&" ".repeat(indent));
    output.push_str(line.trim_end());
    output.push('\n');
}

fn looks_like_module_constant(name: &str) -> bool {
    name.starts_with("__")
        || name
            .chars()
            .all(|ch| ch.is_ascii_uppercase() || ch.is_ascii_digit() || ch == '_')
}

fn is_simple_identifier(name: &str) -> bool {
    let mut chars = name.chars();
    match chars.next() {
        Some(ch) if ch == '_' || ch.is_ascii_alphabetic() => {}
        _ => return false,
    }

    chars.all(|ch| ch == '_' || ch.is_ascii_alphanumeric())
}

fn top_level_constant_items(source: &str) -> Vec<(usize, String)> {
    source
        .lines()
        .enumerate()
        .filter_map(|(row, line)| {
            let trimmed = line.trim();
            if trimmed.is_empty()
                || line.starts_with(' ')
                || line.starts_with('\t')
                || trimmed.starts_with('#')
                || trimmed.starts_with("def ")
                || trimmed.starts_with("async def ")
                || trimmed.starts_with("class ")
                || trimmed.starts_with('@')
                || trimmed.starts_with("\"\"\"")
                || trimmed.starts_with("'''")
            {
                return None;
            }

            if let Some((left, _)) = trimmed.split_once('=') {
                if trimmed.starts_with("type ") {
                    return Some((row, format!("{} = ...", left.trim())));
                }

                if let Some((name, ty)) = left.split_once(':') {
                    let name = name.trim();
                    if looks_like_module_constant(name) && is_simple_identifier(name) {
                        return Some((row, format!("{name}: {} = ...", ty.trim())));
                    }
                }

                let name = left.trim();
                if looks_like_module_constant(name) && is_simple_identifier(name) {
                    return Some((row, format!("{name} = ...")));
                }
            }

            None
        })
        .collect()
}

fn slice(source: &str, start: usize, end: usize) -> &str {
    &source[start..end]
}

fn normalize_newlines(source: &str) -> String {
    source.replace("\r\n", "\n").replace('\r', "\n")
}

fn parse_failure() -> SkeletonOutput {
    SkeletonOutput {
        fence_label: "python",
        body: "...\n".to_string(),
        is_placeholder: false,
    }
}

#[cfg(test)]
mod tests {
    use super::skeletonize;
    use std::path::Path;

    #[test]
    fn extracts_functions_classes_docstrings_and_constants() {
        let source = r#"
"""Module docs."""

FOO = 1
value = 2

@decorator
def work(x: int, y: str = "a"):
    """Work docs."""
    return x

class Service(Base):
    """Service docs."""

    @classmethod
    def build(cls):
        """Build docs."""
        return cls()
"#;

        let output = skeletonize(Path::new("pkg/module.py"), source);

        assert_eq!(output.fence_label, "python");
        assert!(!output.is_placeholder);
        assert_eq!(
            output.body,
            concat!(
                "\"\"\"Module docs.\"\"\"\n",
                "\n",
                "FOO = ...\n",
                "\n",
                "@decorator\n",
                "def work(x: int, y: str = \"a\"): ...\n",
                "    \"\"\"Work docs.\"\"\"\n",
                "\n",
                "class Service(Base):\n",
                "    \"\"\"Service docs.\"\"\"\n",
                "\n",
                "    @classmethod\n",
                "    def build(cls): ...\n",
                "        \"\"\"Build docs.\"\"\"\n",
            )
        );
    }

    #[test]
    fn emits_type_alias_and_annotated_constant() {
        let source = r#"
COUNT: int = 3
type Name = str
"#;

        let output = skeletonize(Path::new("pkg/types.py"), source);

        assert_eq!(output.body, "COUNT: int = ...\n\ntype Name = ...\n");
    }
}
