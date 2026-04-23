use std::path::Path;

use super::SkeletonOutput;
use tree_sitter::{Node, Parser};

pub fn skeletonize(_path: &Path, source: &str) -> SkeletonOutput {
    let normalized = normalize_newlines(source);
    let mut parser = Parser::new();
    parser
        .set_language(&tree_sitter_javascript::LANGUAGE.into())
        .expect("javascript grammar should load");

    let Some(tree) = parser.parse(&normalized, None) else {
        return parse_failure();
    };

    let mut output = String::new();
    render_program(tree.root_node(), &normalized, &mut output);

    if output.is_empty() {
        output.push_str("...\n");
    }

    SkeletonOutput {
        fence_label: "javascript",
        body: output,
        is_placeholder: false,
    }
}

fn render_program(root: Node<'_>, source: &str, output: &mut String) {
    let mut cursor = root.walk();
    let children: Vec<_> = root.named_children(&mut cursor).collect();

    for (idx, child) in children.iter().enumerate() {
        if let Some(block) = render_top_level(*child, source, 0, "") {
            let doc = preceding_jsdoc(&children, idx, source);
            push_block(output, &with_doc(doc, block));
        }
    }
}

fn render_top_level(node: Node<'_>, source: &str, indent: usize, prefix: &str) -> Option<String> {
    match node.kind() {
        "function_declaration" | "generator_function_declaration" => {
            Some(render_function(node, source, indent, prefix))
        }
        "class_declaration" => Some(render_class(node, source, indent, prefix)),
        "lexical_declaration" | "variable_declaration" => {
            render_lexical(node, source, indent, prefix)
        }
        "export_statement" => render_export(node, source, indent),
        _ => None,
    }
}

fn render_export(node: Node<'_>, source: &str, indent: usize) -> Option<String> {
    let prefix = if has_default_token(node, source) {
        "export default "
    } else {
        "export "
    };

    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        if let Some(rendered) = render_top_level(child, source, indent, prefix) {
            return Some(rendered);
        }
    }

    let literal = slice(source, node.start_byte(), node.end_byte()).trim();
    let collapsed = literal.split_whitespace().collect::<Vec<_>>().join(" ");
    let mut rendered = String::new();
    push_line(&mut rendered, indent, &collapsed);
    Some(rendered)
}

fn has_default_token(node: Node<'_>, source: &str) -> bool {
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if !child.is_named() && slice(source, child.start_byte(), child.end_byte()) == "default" {
            return true;
        }
    }
    false
}

fn render_function(node: Node<'_>, source: &str, indent: usize, prefix: &str) -> String {
    let body = node.child_by_field_name("body");
    let header_end = body.map(|b| b.start_byte()).unwrap_or(node.end_byte());
    let header = slice(source, node.start_byte(), header_end)
        .trim_end()
        .to_string();
    let mut rendered = String::new();
    if body.is_some() {
        push_line(
            &mut rendered,
            indent,
            &format!("{prefix}{header} {{ ... }}"),
        );
    } else {
        let literal = slice(source, node.start_byte(), node.end_byte())
            .trim()
            .to_string();
        push_line(&mut rendered, indent, &format!("{prefix}{literal}"));
    }
    rendered
}

fn render_class(node: Node<'_>, source: &str, indent: usize, prefix: &str) -> String {
    let body = node.child_by_field_name("body");
    let header_end = body.map(|b| b.start_byte()).unwrap_or(node.end_byte());
    let header = slice(source, node.start_byte(), header_end)
        .trim_end()
        .to_string();
    let mut rendered = String::new();
    push_line(&mut rendered, indent, &format!("{prefix}{header} {{"));
    if let Some(body) = body {
        render_class_body(body, source, indent + 2, &mut rendered);
    }
    push_line(&mut rendered, indent, "}");
    rendered
}

fn render_class_body(body: Node<'_>, source: &str, indent: usize, output: &mut String) {
    let mut cursor = body.walk();
    let members: Vec<_> = body.named_children(&mut cursor).collect();
    let mut first = true;
    for (idx, member) in members.iter().enumerate() {
        let rendered = match member.kind() {
            "method_definition" => Some(render_method(*member, source, indent)),
            "field_definition" => Some(render_field(*member, source, indent)),
            _ => None,
        };
        if let Some(block) = rendered {
            if !first {
                output.push('\n');
            }
            first = false;
            if let Some(doc) = preceding_jsdoc(&members, idx, source) {
                format_jsdoc(&doc, indent, output);
            }
            output.push_str(&block);
        }
    }
}

fn render_method(node: Node<'_>, source: &str, indent: usize) -> String {
    let body = node.child_by_field_name("body");
    let header_end = body.map(|b| b.start_byte()).unwrap_or(node.end_byte());
    let header = slice(source, node.start_byte(), header_end)
        .trim_end()
        .to_string();
    let mut rendered = String::new();
    if body.is_some() {
        push_line(&mut rendered, indent, &format!("{header} {{ ... }}"));
    } else {
        let literal = slice(source, node.start_byte(), node.end_byte())
            .trim()
            .to_string();
        push_line(&mut rendered, indent, &literal);
    }
    rendered
}

fn render_field(node: Node<'_>, source: &str, indent: usize) -> String {
    let value = node.child_by_field_name("value");
    let header_end = value.map(|v| v.start_byte()).unwrap_or(node.end_byte());
    let header = slice(source, node.start_byte(), header_end)
        .trim_end()
        .trim_end_matches(';')
        .trim_end()
        .trim_end_matches('=')
        .trim_end()
        .to_string();
    let mut rendered = String::new();
    if value.is_some() {
        push_line(&mut rendered, indent, &format!("{header} = ...;"));
    } else {
        push_line(&mut rendered, indent, &format!("{header};"));
    }
    rendered
}

fn render_lexical(node: Node<'_>, source: &str, indent: usize, prefix: &str) -> Option<String> {
    let first_child = node.child(0)?;
    let keyword = slice(source, first_child.start_byte(), first_child.end_byte()).to_string();
    if !matches!(keyword.as_str(), "const" | "let" | "var") {
        return None;
    }

    let mut rendered = String::new();
    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        if child.kind() != "variable_declarator" {
            continue;
        }
        let value = child.child_by_field_name("value");
        let header_end = value.map(|v| v.start_byte()).unwrap_or(child.end_byte());
        let header = slice(source, child.start_byte(), header_end)
            .trim_end()
            .trim_end_matches('=')
            .trim_end()
            .to_string();

        let line = match value.map(|v| (v, v.kind())) {
            Some((v, "arrow_function"))
            | Some((v, "function_expression"))
            | Some((v, "generator_function")) => {
                if let Some(b) = v.child_by_field_name("body") {
                    let fn_prefix = slice(source, v.start_byte(), b.start_byte())
                        .trim_end()
                        .to_string();
                    format!("{prefix}{keyword} {header} = {fn_prefix} {{ ... }};")
                } else {
                    format!("{prefix}{keyword} {header} = ...;")
                }
            }
            Some(_) => format!("{prefix}{keyword} {header} = ...;"),
            None => format!("{prefix}{keyword} {header};"),
        };
        push_line(&mut rendered, indent, &line);
    }

    if rendered.is_empty() {
        None
    } else {
        Some(rendered)
    }
}

fn preceding_jsdoc(siblings: &[Node<'_>], idx: usize, source: &str) -> Option<String> {
    if idx == 0 {
        return None;
    }
    let prev = siblings[idx - 1];
    let curr = siblings[idx];
    if prev.kind() != "comment" {
        return None;
    }
    let text = slice(source, prev.start_byte(), prev.end_byte());
    if !(text.starts_with("/**") && text.ends_with("*/")) {
        return None;
    }
    if curr
        .start_position()
        .row
        .saturating_sub(prev.end_position().row)
        > 1
    {
        return None;
    }
    Some(text.to_string())
}

fn format_jsdoc(doc: &str, indent: usize, output: &mut String) {
    for (i, line) in doc.lines().enumerate() {
        let trimmed = line.trim();
        let shaped = if i == 0 {
            trimmed.to_string()
        } else if trimmed.starts_with('*') {
            format!(" {trimmed}")
        } else {
            trimmed.to_string()
        };
        push_line(output, indent, &shaped);
    }
}

fn with_doc(doc: Option<String>, body: String) -> String {
    match doc {
        Some(doc) => {
            let mut out = String::new();
            format_jsdoc(&doc, 0, &mut out);
            out.push_str(&body);
            out
        }
        None => body,
    }
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

fn slice(source: &str, start: usize, end: usize) -> &str {
    &source[start..end]
}

fn normalize_newlines(source: &str) -> String {
    source.replace("\r\n", "\n").replace('\r', "\n")
}

fn parse_failure() -> SkeletonOutput {
    SkeletonOutput {
        fence_label: "javascript",
        body: "...\n".to_string(),
        is_placeholder: false,
    }
}

#[cfg(test)]
mod tests {
    use super::skeletonize;
    use std::path::Path;

    #[test]
    fn emits_function_and_const() {
        let source = "const COUNT = 3;\nfunction run(x) { return x; }\n";
        let output = skeletonize(Path::new("app.js"), source);
        assert_eq!(output.fence_label, "javascript");
        assert!(!output.is_placeholder);
        assert_eq!(
            output.body,
            "const COUNT = ...;\n\nfunction run(x) { ... }\n"
        );
    }

    #[test]
    fn emits_arrow_const_with_jsdoc() {
        let source = "/** doc */\nexport const add = (a, b) => a + b;\n";
        let output = skeletonize(Path::new("app.js"), source);
        assert_eq!(
            output.body,
            "/** doc */\nexport const add = (a, b) => { ... };\n"
        );
    }
}
