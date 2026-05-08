#![allow(dead_code)]

mod dead;
mod live;

pub use dead::DeadWire;
pub use live::{wire_tag, LiveWire};
