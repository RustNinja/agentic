mod dead;
mod live;

pub use dead::{dead_patch, DeadPatch};
pub use live::{selected_patch, PatchError, PatchOp, PatchRequest, PatchSegment};
