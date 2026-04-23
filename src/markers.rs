use std::error::Error;
use std::fmt;

pub const START_MARKER: &str = "<!-- frugal:managed:start -->";
pub const END_MARKER: &str = "<!-- frugal:managed:end -->";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MarkerError {
    MissingEndMarker,
    MisorderedMarkers,
    MultipleManagedBlocks,
}

impl fmt::Display for MarkerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingEndMarker => write!(f, "missing managed block end marker"),
            Self::MisorderedMarkers => write!(f, "managed block end marker appears before start"),
            Self::MultipleManagedBlocks => write!(f, "multiple managed blocks are not supported"),
        }
    }
}

impl Error for MarkerError {}

pub fn render_managed_block(body: &str) -> String {
    let normalized_body = normalize_body(body);

    format!("{START_MARKER}\n{normalized_body}{END_MARKER}\n")
}

pub fn upsert_managed_block(input: &str, body: &str) -> Result<String, MarkerError> {
    let block = render_managed_block(body);
    let start = input.find(START_MARKER);
    let end = input.find(END_MARKER);

    match (start, end) {
        (None, None) => Ok(insert_block(input, &block)),
        (Some(_), None) => Err(MarkerError::MissingEndMarker),
        (None, Some(_)) => Err(MarkerError::MisorderedMarkers),
        (Some(start_idx), Some(end_idx)) => replace_block(input, start_idx, end_idx, &block),
    }
}

fn replace_block(
    input: &str,
    start_idx: usize,
    end_idx: usize,
    block: &str,
) -> Result<String, MarkerError> {
    if end_idx < start_idx {
        return Err(MarkerError::MisorderedMarkers);
    }

    let after_end_idx = end_idx + END_MARKER.len();
    if input[after_end_idx..].contains(START_MARKER) || input[after_end_idx..].contains(END_MARKER) {
        return Err(MarkerError::MultipleManagedBlocks);
    }
    if input[..start_idx].contains(START_MARKER) || input[..start_idx].contains(END_MARKER) {
        return Err(MarkerError::MultipleManagedBlocks);
    }

    let mut suffix_start = after_end_idx;
    if input[suffix_start..].starts_with("\r\n") {
        suffix_start += 2;
    } else if input[suffix_start..].starts_with('\n') {
        suffix_start += 1;
    }

    let mut output = String::with_capacity(input.len() - (suffix_start - start_idx) + block.len());
    output.push_str(&input[..start_idx]);
    output.push_str(block);
    output.push_str(&input[suffix_start..]);
    Ok(output)
}

fn insert_block(input: &str, block: &str) -> String {
    if input.is_empty() {
        return block.to_string();
    }

    let mut output = String::with_capacity(input.len() + block.len() + 2);
    output.push_str(input);

    if !input.ends_with('\n') {
        output.push('\n');
    }
    if !output.ends_with("\n\n") {
        output.push('\n');
    }

    output.push_str(block);
    output
}

fn normalize_body(body: &str) -> String {
    let normalized = body.replace("\r\n", "\n");

    if normalized.is_empty() {
        return String::new();
    }

    if normalized.ends_with('\n') {
        normalized
    } else {
        format!("{normalized}\n")
    }
}

#[cfg(test)]
mod tests {
    use super::{upsert_managed_block, END_MARKER, START_MARKER};

    #[test]
    fn insert_missing_managed_block() {
        let input = "# AGENTS\n";
        let output = upsert_managed_block(input, "managed body").expect("insert should succeed");

        assert_eq!(
            output,
            format!(
                "# AGENTS\n\n{START_MARKER}\nmanaged body\n{END_MARKER}\n"
            )
        );
    }

    #[test]
    fn replace_existing_managed_block() {
        let input = format!(
            "# AGENTS\n\n{START_MARKER}\nold body\n{END_MARKER}\n"
        );

        let output = upsert_managed_block(&input, "new body").expect("replace should succeed");

        assert_eq!(
            output,
            format!(
                "# AGENTS\n\n{START_MARKER}\nnew body\n{END_MARKER}\n"
            )
        );
    }

    #[test]
    fn preserve_text_outside_markers() {
        let input = format!(
            "before\n\n{START_MARKER}\nold\n{END_MARKER}\n\nafter\n"
        );

        let output = upsert_managed_block(&input, "new").expect("replace should succeed");

        assert_eq!(
            output,
            format!("before\n\n{START_MARKER}\nnew\n{END_MARKER}\n\nafter\n")
        );
    }
}
