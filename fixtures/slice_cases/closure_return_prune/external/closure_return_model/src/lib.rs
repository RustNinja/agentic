mod dead;
mod live;

pub use dead::{dead_closure_return, DeadClosureReturn};
pub use live::{selected_closure_return, ClosureReturnRecord};
