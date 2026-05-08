mod dead;
mod live;

pub use dead::{dead_const, DeadConstRecord};
pub use live::{selected_const, ConstRecord, BASE, LABEL, SCALE};
