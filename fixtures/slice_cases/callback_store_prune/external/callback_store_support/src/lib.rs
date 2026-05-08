mod dead;
mod live;

pub use dead::{dead_callback_store, DeadCallbackStore};
pub use live::{selected_callback_store, CallbackStore, EventRecord, EventSink};
