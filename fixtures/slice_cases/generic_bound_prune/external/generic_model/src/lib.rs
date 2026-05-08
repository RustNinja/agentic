mod dead;
mod live;

pub use dead::{dead_generic, DeadGeneric};
pub use live::{render_with_bound, GenericRecord, LabelRender, selected_generic};
