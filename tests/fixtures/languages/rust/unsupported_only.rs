use std::collections::BTreeMap;

macro_rules! make_thing {
    () => {};
}

mod nested {
    pub fn hidden() {}
}

extern crate core;
