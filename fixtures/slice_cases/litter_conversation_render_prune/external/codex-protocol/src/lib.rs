#![allow(dead_code)]

mod dead;
mod live;

pub use dead::{dead_event, DeadEvent};
pub use live::{ConversationEvent, Message, Role};

