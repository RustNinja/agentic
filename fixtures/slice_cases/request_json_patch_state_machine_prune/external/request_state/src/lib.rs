mod dead;
mod live;

pub use dead::{dead_state_summary, DeadState};
pub use live::{apply_patch_request, PatchCommand, PatchError, PatchRequest};
