mod dead;
mod live;

pub use dead::{dead_deref, DeadDeref};
pub use live::{selected_deref, DerefInner, DerefWrapper};
