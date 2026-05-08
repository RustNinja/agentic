#![allow(dead_code)]

mod dead;
mod live;

pub use dead::{dead_stream, DeadEnvelope, DeadEvent};
pub use live::{event_iter, selected_stream, EventDto, EventEnvelope, StreamError, StreamResult};
