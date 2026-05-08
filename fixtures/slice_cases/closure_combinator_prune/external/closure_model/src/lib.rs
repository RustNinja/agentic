mod dead;
mod live;

pub use dead::{dead_closure, DeadClosure};
pub use live::{selected_closure, CleanError, CleanItem, RawItem};
