mod dead;
mod live;

pub use dead::{dead_dispatch, DeadDispatch, StopParams};
pub use live::{selected_dispatch, DispatchMethod, StartParams};
