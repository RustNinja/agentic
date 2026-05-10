#![allow(dead_code)]

mod dead;
mod live;

pub use dead::{dead_protocol_codegen_report, DeadGeneratedProtocol};
pub use live::{selected_wire_event, GeneratedEvent};

