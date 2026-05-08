mod dead;
mod live;

pub use dead::{dead_array, DeadArray, DEAD_LEN};
pub use live::{selected_array, ArrayPayload, LIVE_LEN};
