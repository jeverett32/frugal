use std::path::Path;

use super::{placeholder_body, SkeletonOutput};

pub fn skeletonize(path: &Path, _source: &str) -> SkeletonOutput {
    SkeletonOutput {
        fence_label: "text",
        body: placeholder_body("rust", path),
        is_placeholder: true,
    }
}
