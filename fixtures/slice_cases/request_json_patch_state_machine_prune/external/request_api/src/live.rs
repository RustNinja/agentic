pub use request_state::PatchError;

pub fn selected_patch_summary(raw: &str) -> Result<String, PatchError> {
    request_state::apply_patch_request(raw)
}

pub fn dead_live_patch_summary(raw: &str) -> String {
    format!("dead-live:{raw}")
}
