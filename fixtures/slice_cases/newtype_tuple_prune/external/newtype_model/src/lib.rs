mod dead;
mod live;

pub use dead::{dead_newtype, DeadNewtype};
pub use live::{selected_newtype, NewtypeInner, NewtypeRecord};
