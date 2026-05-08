#![allow(dead_code)]

mod dead;
mod live;

pub use dead::{dead_callback_fn, DeadCallback, DeadInput};
pub use live::{apply_callback, bump_callback, Callback, CallbackInput};
