pub fn estimate_tokens(bytes: usize) -> usize {
    bytes.div_ceil(4)
}

#[cfg(test)]
mod tests {
    use super::estimate_tokens;

    #[test]
    fn rounds_up_bytes_divided_by_four() {
        assert_eq!(estimate_tokens(0), 0);
        assert_eq!(estimate_tokens(1), 1);
        assert_eq!(estimate_tokens(4), 1);
        assert_eq!(estimate_tokens(5), 2);
        assert_eq!(estimate_tokens(21), 6);
        assert_eq!(estimate_tokens(46), 12);
    }
}
