use std::path::Path;

mod go;
mod javascript;
mod python;
mod rust;
mod typescript;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SkeletonOutput {
    pub fence_label: &'static str,
    pub body: String,
    pub is_placeholder: bool,
}

pub fn is_secondary_eligible(path: &Path) -> bool {
    !matches!(language_for_path(path), Language::Unknown)
}

pub fn skeletonize(path: &Path, source: &str) -> SkeletonOutput {
    match language_for_path(path) {
        Language::Python => python::skeletonize(path, source),
        Language::Rust => rust::skeletonize(path, source),
        Language::JavaScript => javascript::skeletonize(path, source),
        Language::TypeScript => typescript::skeletonize(path, source),
        Language::Go => go::skeletonize(path, source),
        Language::Unknown => unknown_placeholder(path),
    }
}

pub fn fence_label_for_path(path: &Path) -> Option<&'static str> {
    match path.extension().and_then(|ext| ext.to_str()) {
        Some("md") => Some("markdown"),
        Some("rs") => Some("rust"),
        Some("py") => Some("python"),
        Some("js" | "jsx" | "mjs" | "cjs") => Some("javascript"),
        Some("ts" | "tsx" | "mts" | "cts") => Some("typescript"),
        Some("go") => Some("go"),
        Some("toml") => Some("toml"),
        Some("json") => Some("json"),
        Some("yml" | "yaml") => Some("yaml"),
        Some("txt") => Some("text"),
        _ => None,
    }
}

pub(crate) fn placeholder_body(language: &str, path: &Path) -> String {
    format!(
        "TODO: {language} skeleton placeholder\npath: {}\nstatus: deterministic placeholder; extraction not implemented\n",
        path.display()
    )
}

fn unknown_placeholder(path: &Path) -> SkeletonOutput {
    SkeletonOutput {
        fence_label: "text",
        body: placeholder_body("unknown", path),
        is_placeholder: true,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Language {
    Python,
    Rust,
    JavaScript,
    TypeScript,
    Go,
    Unknown,
}

pub fn language_for_path(path: &Path) -> Language {
    match path.extension().and_then(|ext| ext.to_str()) {
        Some("py") => Language::Python,
        Some("rs") => Language::Rust,
        Some("js" | "jsx" | "mjs" | "cjs") => Language::JavaScript,
        Some("ts" | "tsx" | "mts" | "cts") => Language::TypeScript,
        Some("go") => Language::Go,
        _ => Language::Unknown,
    }
}

impl Language {
    pub fn label(self) -> Option<&'static str> {
        match self {
            Self::Python => Some("python"),
            Self::Rust => Some("rust"),
            Self::JavaScript => Some("javascript"),
            Self::TypeScript => Some("typescript"),
            Self::Go => Some("go"),
            Self::Unknown => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn rust_skeleton_depends_on_source() {
        let path = PathBuf::from("src/lib.rs");
        let left = skeletonize(&path, "fn left() {}\n");
        let right = skeletonize(&path, "fn right() {}\n");

        assert_eq!(left.fence_label, "rust");
        assert_eq!(right.fence_label, "rust");
        assert!(!left.is_placeholder);
        assert!(!right.is_placeholder);
        assert_ne!(left.body, right.body);
    }

    #[test]
    fn javascript_skeleton_depends_on_source() {
        let path = PathBuf::from("web/app.js");
        let left = skeletonize(&path, "function left() {}\n");
        let right = skeletonize(&path, "function right() {}\n");

        assert_eq!(left.fence_label, "javascript");
        assert_eq!(right.fence_label, "javascript");
        assert!(!left.is_placeholder);
        assert!(!right.is_placeholder);
        assert_ne!(left.body, right.body);
    }

    #[test]
    fn typescript_skeleton_depends_on_source() {
        let path = PathBuf::from("web/app.ts");
        let left = skeletonize(&path, "export type Left = string;\n");
        let right = skeletonize(&path, "export type Right = number;\n");

        assert_eq!(left.fence_label, "typescript");
        assert_eq!(right.fence_label, "typescript");
        assert!(!left.is_placeholder);
        assert!(!right.is_placeholder);
        assert_ne!(left.body, right.body);
    }

    #[test]
    fn go_skeleton_depends_on_source() {
        let path = PathBuf::from("cmd/main.go");
        let left = skeletonize(&path, "package main\nfunc left() {}\n");
        let right = skeletonize(&path, "package main\nfunc right() {}\n");

        assert_eq!(left.fence_label, "go");
        assert_eq!(right.fence_label, "go");
        assert!(!left.is_placeholder);
        assert!(!right.is_placeholder);
        assert_ne!(left.body, right.body);
    }

    #[test]
    fn python_skeleton_depends_on_source() {
        let path = PathBuf::from("pkg/module.py");
        let left = skeletonize(&path, "def left():\n    pass\n");
        let right = skeletonize(&path, "def right():\n    return 1\n");

        assert_eq!(left.fence_label, "python");
        assert_eq!(right.fence_label, "python");
        assert!(!left.is_placeholder);
        assert!(!right.is_placeholder);
        assert_ne!(left.body, right.body);
    }
}
