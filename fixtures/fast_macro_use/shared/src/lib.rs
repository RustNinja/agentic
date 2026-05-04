pub type SharedAlias = u32;

pub const FEATURE_FLAG: SharedAlias = 11;

pub static SHARED_STATIC: SharedAlias = 13;

pub struct SharedRecord {
    pub value: SharedAlias,
}

impl SharedRecord {
    pub fn new(value: SharedAlias) -> Self {
        Self { value }
    }
}

pub enum SharedMode {
    Fast(SharedAlias),
    Slow,
}

impl SharedMode {
    pub fn score(&self) -> SharedAlias {
        match self {
            SharedMode::Fast(value) => *value,
            SharedMode::Slow => 1,
        }
    }
}

pub fn fixture_value() -> SharedAlias {
    nested::nested_value() + helper_marker()
}

pub fn helper_marker() -> SharedAlias {
    3
}

pub fn dead_shared() -> SharedAlias {
    99
}

pub enum DeadEnum {
    Dead,
}

pub mod nested {
    pub fn nested_value() -> crate::SharedAlias {
        7
    }

    pub fn dead_nested() -> crate::SharedAlias {
        99
    }
}

pub mod prelude {
    pub use crate::nested::nested_value as exported_nested;
    pub use crate::{SharedAlias, SharedMode, FEATURE_FLAG};

    pub fn dead_prelude() -> SharedAlias {
        99
    }
}

#[macro_export]
macro_rules! shared_macro {
    ($value:expr) => {
        $crate::helper_marker() + $value
    };
}
