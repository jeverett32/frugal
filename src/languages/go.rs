use std::path::Path;

use super::SkeletonOutput;
use tree_sitter::{Node, Parser};

pub fn skeletonize(_path: &Path, source: &str) -> SkeletonOutput {
    let normalized = normalize_newlines(source);
    let mut parser = Parser::new();
    parser
        .set_language(&tree_sitter_go::LANGUAGE.into())
        .expect("go grammar should load");

    let Some(tree) = parser.parse(&normalized, None) else {
        return parse_failure();
    };

    let mut output = String::new();
    render_source(tree.root_node(), &normalized, &mut output);

    if output.is_empty() {
        output.push_str("...\n");
    }

    SkeletonOutput {
        fence_label: "go",
        body: output,
        is_placeholder: false,
    }
}

fn render_source(root: Node<'_>, source: &str, output: &mut String) {
    let mut cursor = root.walk();
    let children = root.named_children(&mut cursor).collect::<Vec<_>>();
    let mut items = Vec::new();

    for child in children {
        let rendered = match child.kind() {
            "package_clause" => render_package(child, source),
            "function_declaration" | "method_declaration" => render_function_like(child, source),
            "type_declaration" => render_type_declaration(child, source),
            "const_declaration" => render_value_declaration("const", child, source),
            "var_declaration" => render_value_declaration("var", child, source),
            _ => continue,
        };

        items.push((child.start_position().row, rendered));
    }

    items.sort_by_key(|(row, _)| *row);

    for (_, block) in items {
        push_block(output, &block);
    }
}

fn render_package(node: Node<'_>, source: &str) -> String {
    slice(source, node.start_byte(), node.end_byte())
        .trim_end()
        .to_string()
}

fn render_function_like(node: Node<'_>, source: &str) -> String {
    let mut rendered = String::new();
    push_attached_comment(
        &mut rendered,
        attached_comment(source, node.start_position().row),
        0,
    );

    let end = node
        .child_by_field_name("body")
        .map(|body| body.start_byte())
        .unwrap_or_else(|| node.end_byte());
    let mut header = slice(source, node.start_byte(), end).trim_end().to_string();

    if node.child_by_field_name("body").is_some() {
        header.push_str(" { ... }");
    }

    push_line(&mut rendered, 0, &header);
    rendered
}

fn render_type_declaration(node: Node<'_>, source: &str) -> String {
    let mut rendered = String::new();
    push_attached_comment(
        &mut rendered,
        attached_comment(source, node.start_position().row),
        0,
    );
    rendered.push_str(slice(source, node.start_byte(), node.end_byte()).trim_end());
    rendered.push('\n');
    rendered
}

fn render_value_declaration(keyword: &str, node: Node<'_>, source: &str) -> String {
    let mut rendered = String::new();
    push_attached_comment(
        &mut rendered,
        attached_comment(source, node.start_position().row),
        0,
    );
    let raw = slice(source, node.start_byte(), node.end_byte()).trim_end();

    let first_line = raw.lines().next().unwrap_or_default().trim_end();
    if first_line.ends_with('(') {
        let lines = raw.lines().collect::<Vec<_>>();
        push_line(&mut rendered, 0, &format!("{keyword} ("));

        for line in lines.iter().skip(1) {
            let trimmed = line.trim();
            if trimmed == ")" {
                break;
            }
            if trimmed.is_empty() {
                continue;
            }
            if is_comment_line(trimmed) {
                push_line(&mut rendered, 4, trimmed);
                continue;
            }
            push_line(&mut rendered, 4, &collapse_value_line(trimmed));
        }

        push_line(&mut rendered, 0, ")");
        return rendered;
    }

    let collapsed = raw
        .strip_prefix(keyword)
        .map(str::trim)
        .map(collapse_value_line)
        .unwrap_or_else(|| keyword.to_string());
    push_line(&mut rendered, 0, &format!("{keyword} {collapsed}"));
    rendered
}

fn collapse_value_line(line: &str) -> String {
    match line.split_once('=') {
        Some((left, _)) => format!("{} = ...", left.trim_end()),
        None => line.to_string(),
    }
}

fn attached_comment(source: &str, start_row: usize) -> Option<String> {
    if start_row == 0 {
        return None;
    }

    let lines = source.lines().collect::<Vec<_>>();
    if start_row > lines.len() {
        return None;
    }

    let mut row = start_row;
    let mut start = start_row;

    while row > 0 {
        let line = lines[row - 1];
        let trimmed = line.trim();
        if trimmed.is_empty() || !is_comment_line(trimmed) {
            break;
        }
        start = row - 1;
        row -= 1;
    }

    if start == start_row {
        return None;
    }

    Some(lines[start..start_row].join("\n"))
}

fn is_comment_line(line: &str) -> bool {
    line.starts_with("//")
        || line.starts_with("/*")
        || line.starts_with('*')
        || line.starts_with("*/")
}

fn push_attached_comment(output: &mut String, comment: Option<String>, indent: usize) {
    let Some(comment) = comment else {
        return;
    };

    for line in comment.lines() {
        let trimmed = line.trim_start();
        if trimmed.is_empty() {
            continue;
        }
        push_line(output, indent, trimmed);
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
        fence_label: "go",
        body: "...\n".to_string(),
        is_placeholder: false,
    }
}

#[cfg(test)]
mod tests {
    use super::skeletonize;
    use std::path::Path;

    #[test]
    fn extracts_package_decls_comments_and_signatures() {
        let source = r#"package sample

import "fmt"

const (
    // Status docs.
    statusReady = "ready"
    statusDone  = "done"
)

var version = buildVersion()

// User docs.
type User struct {
    ID   int
    Name string
}

// Reader docs.
type Reader interface {
    Read(p []byte) (n int, err error)
}

type ID = string

// NewUser docs.
func NewUser(name string) *User {
    return &User{Name: name}
}

// NameLen docs.
func (u *User) NameLen() int {
    return len(u.Name)
}
"#;

        let output = skeletonize(Path::new("pkg/sample.go"), source);

        assert_eq!(output.fence_label, "go");
        assert!(!output.is_placeholder);
        assert_eq!(
            output.body,
            concat!(
                "package sample\n",
                "\n",
                "const (\n",
                "    // Status docs.\n",
                "    statusReady = ...\n",
                "    statusDone = ...\n",
                ")\n",
                "\n",
                "var version = ...\n",
                "\n",
                "// User docs.\n",
                "type User struct {\n",
                "    ID   int\n",
                "    Name string\n",
                "}\n",
                "\n",
                "// Reader docs.\n",
                "type Reader interface {\n",
                "    Read(p []byte) (n int, err error)\n",
                "}\n",
                "\n",
                "type ID = string\n",
                "\n",
                "// NewUser docs.\n",
                "func NewUser(name string) *User { ... }\n",
                "\n",
                "// NameLen docs.\n",
                "func (u *User) NameLen() int { ... }\n",
            )
        );
    }
}
