mod dead;
mod live;

pub use dead::{dead_free_closure, DeadFreeClosure};
pub use live::{with_free_payload, FreePayload};
