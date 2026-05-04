pub struct SharedRecord {
    pub value: u32,
}

impl SharedRecord {
    pub fn new(value: u32) -> Self {
        Self { value }
    }
}

pub fn fixture_value() -> u32 {
    nested::nested_value() + helper_marker()
}

pub fn helper_marker() -> u32 {
    3
}

pub fn dead_shared() -> u32 {
    99
}

pub mod nested {
    pub fn nested_value() -> u32 {
        7
    }

    pub fn dead_nested() -> u32 {
        99
    }
}

#[macro_export]
macro_rules! shared_macro {
    ($value:expr) => {
        $crate::helper_marker() + $value
    };
}
