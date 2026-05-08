mod dead;
mod live;

pub use dead::{dead_mutex, DeadMutexEntry};
pub use live::{selected_mutex, MutexEntry};
