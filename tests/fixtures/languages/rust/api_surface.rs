use std::fmt::Debug;

/// Adds two numbers.
pub fn add(left: i32, right: i32) -> i32 {
    left + right
}

/// Point docs.
#[derive(Debug)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

/// Result docs.
pub enum Status {
    Ready,
    Busy(u32),
}

/// Alias docs.
pub type ResultMap<T> = std::collections::BTreeMap<String, T>;

/// Limit docs.
pub const LIMIT: usize = 128;

/// Cache docs.
pub static mut CACHE: Option<String> = None;

/// Trait docs.
pub trait Worker {
    /// Name docs.
    fn name(&self) -> &str;

    /// Run docs.
    fn run(&self) {
        panic!("not used");
    }

    type Error;
}

impl Point {
    /// Origin docs.
    pub fn origin() -> Self {
        Self { x: 0, y: 0 }
    }

    const fn hidden() -> usize {
        1
    }

    const SCALE: usize = 2;
}
