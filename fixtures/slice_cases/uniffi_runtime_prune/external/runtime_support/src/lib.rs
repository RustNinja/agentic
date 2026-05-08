mod dead;
mod live;

pub use dead::{dead_runtime, DeadRuntimeObject};
pub use live::{selected_runtime, shared_runtime, RuntimeObject};
