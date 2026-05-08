mod dead;
mod live;

pub use dead::{dead_guard, DeadGuard};
pub use live::{selected_guard, GuardRecord, GuardState};
