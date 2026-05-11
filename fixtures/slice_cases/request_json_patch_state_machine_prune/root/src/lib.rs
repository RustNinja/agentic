use opensourced::opensourced;

#[opensourced]
pub fn selected_patch_summary(raw: &str) -> Result<String, request_api::PatchError> {
    request_api::selected_patch_summary(raw)
}

pub fn dead_patch_summary(raw: &str) -> String {
    request_api::dead_patch_summary(raw)
}
