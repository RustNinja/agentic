#![allow(dead_code)]

mod dead;
mod live;

pub use dead::{dead_bridge, DeadAdapter};
pub use live::{selected_bridge, AdapterRecord};
