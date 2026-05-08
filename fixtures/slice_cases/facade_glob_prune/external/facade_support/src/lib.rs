#![allow(dead_code)]

mod dead;
mod live;

pub mod facade {
    pub mod nested {
        pub use crate::dead::{dead_factory, DeadRecord};
        pub use crate::live::{build_live, LiveRecord};
    }

    pub use nested::*;
}

pub use facade::*;
