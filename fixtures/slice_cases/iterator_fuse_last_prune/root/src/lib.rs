use opensourced::opensourced;

#[opensourced]
pub fn selected_fuse_last_report(raw: &str) -> String {
    fuse_last_api::selected_fuse_last_report(raw)
}

pub fn dead_fuse_last_report(raw: &str) -> String {
    fuse_last_api::dead_fuse_last_report(raw)
}
