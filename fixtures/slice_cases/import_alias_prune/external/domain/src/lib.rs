#![allow(dead_code)]

pub mod dead;
pub mod live;

pub mod prelude {
    pub use crate::dead::dead_prefix;
    pub use crate::live::model_prefix;
}

pub use dead::{build_dead_model, DeadModel};
pub use live::{build_model, LiveModel};
