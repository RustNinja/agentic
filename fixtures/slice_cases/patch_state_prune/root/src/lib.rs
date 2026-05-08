use opensourced::opensourced;

#[opensourced]
pub fn selected_patch_report(raw: &str) -> String {
    patch_api::selected_patch_report(raw)
}

pub fn dead_patch_report(raw: &str) -> String {
    patch_api::dead_patch_report(raw)
}
