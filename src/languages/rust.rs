use std::path::Path;

use tree_sitter::{Node, Parser};

use super::SkeletonOutput;

pub fn skeletonize(_path: &Path, source: &str) -> SkeletonOutput {
    let normalized = normalize_newlines(source);

    let mut parser = Parser::new();
    parser
        .set_language(&tree_sitter_rust::LANGUAGE.into())
        .expect("rust grammar should load");

    let Some(tree) = parser.parse(&normalized, None) else {
        return empty_output();
    };

    let mut output = String::new();
    let mut cursor = tree.root_node().walk();
    for child in tree.root_node().named_children(&mut cursor) {
        if let Some(block) = render_top_level_item(child, &normalized) {
            push_block(&mut output, &block);
        }
    }

    if output.is_empty() {
        output.push_str("...\n");
    } else if !output.ends_with('\n') {
        output.push('\n');
    }

    SkeletonOutput {
        fence_label: "rust",
        body: output,
        is_placeholder: false,
    }
}

fn render_top_level_item(node: Node<'_>, source: &str) -> Option<String> {
    match node.kind() {
        "function_item" => Some(render_function_item(node, source, 0)),
        "struct_item" | "enum_item" | "type_item" => Some(render_item_with_docs(
            node,
            source,
            render_exact_item(node, source, 0)?,
        )),
        "trait_item" => Some(render_trait_item(node, source, 0)),
        "impl_item" => render_impl_item(node, source, 0),
        "const_item" | "static_item" => Some(render_item_with_docs(
            node,
            source,
            render_const_like_item(node, source, 0)?,
        )),
        _ => None,
    }
}

fn render_function_item(node: Node<'_>, source: &str, indent: usize) -> String {
    let body = node
        .child_by_field_name("body")
        .expect("function_item should have body");
    let header = slice(source, node.start_byte(), body.start_byte()).trim_end();

    render_item_with_docs(
        node,
        source,
        indent_block(&format!("{header} {{ ... }}"), indent),
    )
}

fn render_trait_item(node: Node<'_>, source: &str, indent: usize) -> String {
    let body = node
        .child_by_field_name("body")
        .expect("trait_item should have body");
    let header = slice(source, node.start_byte(), body.start_byte()).trim_end();
    let mut rendered = String::new();

    rendered.push_str(&indent_block(&format!("{header} {{"), indent));
    rendered.push('\n');

    let mut body_cursor = body.walk();
    for child in body.named_children(&mut body_cursor) {
        let Some(block) = render_trait_member(child, source, indent + 4) else {
            continue;
        };
        push_nested_block(&mut rendered, &block);
    }

    if !rendered.ends_with('\n') {
        rendered.push('\n');
    }
    rendered.push_str(&" ".repeat(indent));
    rendered.push('}');

    render_item_with_docs(node, source, rendered)
}

fn render_impl_item(node: Node<'_>, source: &str, indent: usize) -> Option<String> {
    let body = node.child_by_field_name("body")?;
    let header = slice(source, node.start_byte(), body.start_byte()).trim_end();
    let mut rendered = String::new();

    rendered.push_str(&indent_block(&format!("{header} {{"), indent));
    rendered.push('\n');

    let mut has_methods = false;
    let mut body_cursor = body.walk();
    for child in body.named_children(&mut body_cursor) {
        let Some(block) = render_impl_member(child, source, indent + 4) else {
            continue;
        };
        push_nested_block(&mut rendered, &block);
        has_methods = true;
    }

    if !has_methods {
        return None;
    }

    if !rendered.ends_with('\n') {
        rendered.push('\n');
    }
    rendered.push_str(&" ".repeat(indent));
    rendered.push('}');

    Some(render_item_with_docs(node, source, rendered))
}

fn render_trait_member(node: Node<'_>, source: &str, indent: usize) -> Option<String> {
    match node.kind() {
        "function_signature_item" => {
            let text = render_exact_item(node, source, indent)?;
            Some(render_item_with_docs(node, source, text))
        }
        "function_item" => Some(render_function_item(node, source, indent)),
        _ => None,
    }
}

fn render_impl_member(node: Node<'_>, source: &str, indent: usize) -> Option<String> {
    match node.kind() {
        "function_item" => Some(render_function_item(node, source, indent)),
        _ => None,
    }
}

fn render_const_like_item(node: Node<'_>, source: &str, indent: usize) -> Option<String> {
    let value = node.child_by_field_name("value");
    let text = if let Some(value) = value {
        let prefix = slice(source, node.start_byte(), value.start_byte()).trim_end();
        format!("{prefix} ...;")
    } else {
        node.utf8_text(source.as_bytes()).ok()?.trim().to_string()
    };

    Some(indent_block(&text, indent))
}

fn render_exact_item(node: Node<'_>, source: &str, indent: usize) -> Option<String> {
    let text = node.utf8_text(source.as_bytes()).ok()?.trim();
    Some(indent_block(text, indent))
}

fn render_item_with_docs(node: Node<'_>, source: &str, rendered_item: String) -> String {
    let docs = attached_doc_comments(node, source);
    if docs.is_empty() {
        return rendered_item;
    }

    let indent = rendered_item
        .lines()
        .next()
        .map(leading_space_count)
        .unwrap_or(0);
    let mut rendered = String::new();
    for line in docs {
        rendered.push_str(&" ".repeat(indent));
        rendered.push_str(&line);
        rendered.push('\n');
    }
    rendered.push_str(&rendered_item);
    rendered
}

fn attached_doc_comments(node: Node<'_>, source: &str) -> Vec<String> {
    let lines = source.lines().collect::<Vec<_>>();
    if lines.is_empty() || node.start_position().row == 0 {
        return Vec::new();
    }

    let mut docs = Vec::new();
    let mut row = node.start_position().row as isize - 1;

    while row >= 0 {
        let trimmed = lines[row as usize].trim();
        if trimmed.is_empty() {
            break;
        }

        if is_attribute_line(trimmed) {
            row -= 1;
            continue;
        }

        if is_line_doc_comment(trimmed) {
            docs.push(trimmed.to_string());
            row -= 1;
            continue;
        }

        if trimmed.ends_with("*/") {
            let Some((next_row, mut block)) = collect_block_doc_comment(&lines, row) else {
                break;
            };
            docs.append(&mut block);
            row = next_row;
            continue;
        }

        break;
    }

    docs.reverse();
    docs
}

fn collect_block_doc_comment(lines: &[&str], end_row: isize) -> Option<(isize, Vec<String>)> {
    let mut row = end_row;
    let mut block = Vec::new();

    while row >= 0 {
        let trimmed = lines[row as usize].trim();
        block.push(trimmed.to_string());

        if trimmed.starts_with("/**") || trimmed.starts_with("/*!") {
            return Some((row - 1, block));
        }

        row -= 1;
    }

    None
}

fn is_line_doc_comment(line: &str) -> bool {
    line.starts_with("///") || line.starts_with("//!")
}

fn is_attribute_line(line: &str) -> bool {
    line.starts_with("#[") || line.starts_with("#![")
}

fn indent_block(block: &str, indent: usize) -> String {
    if indent == 0 {
        return block.to_string();
    }

    block
        .lines()
        .map(|line| format!("{}{}", " ".repeat(indent), line))
        .collect::<Vec<_>>()
        .join("\n")
}

fn push_block(output: &mut String, block: &str) {
    if block.trim().is_empty() {
        return;
    }

    if !output.is_empty() {
        output.push('\n');
        output.push('\n');
    }

    output.push_str(block.trim_end());
}

fn push_nested_block(output: &mut String, block: &str) {
    if block.trim().is_empty() {
        return;
    }

    if !output.ends_with("{\n") {
        output.push('\n');
    }
    output.push('\n');
    output.push_str(block.trim_end());
}

fn normalize_newlines(source: &str) -> String {
    source.replace("\r\n", "\n")
}

fn empty_output() -> SkeletonOutput {
    SkeletonOutput {
        fence_label: "rust",
        body: "...\n".to_string(),
        is_placeholder: false,
    }
}

fn slice(source: &str, start: usize, end: usize) -> &str {
    &source[start..end]
}

fn leading_space_count(line: &str) -> usize {
    line.chars().take_while(|ch| *ch == ' ').count()
}
