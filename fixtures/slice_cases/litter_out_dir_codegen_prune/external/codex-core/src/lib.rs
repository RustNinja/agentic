#![allow(dead_code)]

mod dead;
mod live;

pub use dead::{dead_core_codegen, DeadGeneratedCore};
pub use live::{render_core_generated_event, CoreGeneratedEvent};

