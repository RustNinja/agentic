pub fn selected_patch_report(raw: &str) -> String {
    patch_model::selected_patch(raw)
}

pub fn dead_live_patch_report(raw: &str) -> String {
    format!("dead-live:{raw}")
}
