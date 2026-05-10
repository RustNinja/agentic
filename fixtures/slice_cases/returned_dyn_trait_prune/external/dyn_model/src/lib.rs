mod dead;
mod live;

pub use dead::{dead_dyn_summary, DeadReader};
pub use live::{render_reader, selected_reader, LiveReader, Reader};

