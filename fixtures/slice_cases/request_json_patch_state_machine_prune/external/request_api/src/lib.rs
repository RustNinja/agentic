mod dead;
mod live;

pub use dead::{dead_patch_summary, DeadPatchRequest};
pub use live::{selected_patch_summary, PatchError};
