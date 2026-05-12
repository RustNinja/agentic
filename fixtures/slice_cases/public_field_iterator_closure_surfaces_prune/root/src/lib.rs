use opensourced::opensourced;

#[opensourced]
pub fn selected_summary(seed: u32) -> String {
    field_api::selected_summary(seed)
}

#[allow(dead_code)]
pub fn dead_summary(seed: u32) -> String {
    field_api::dead_summary(seed)
}
