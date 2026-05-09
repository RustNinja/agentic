pub fn selected_fuse_last_report(raw: &str) -> String {
    fuse_last_model::selected_fuse_last(raw)
}

pub fn dead_live_fuse_last_report(raw: &str) -> String {
    format!("dead-fuse-last-live-report:{raw}")
}
