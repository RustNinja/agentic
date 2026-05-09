use opensourced::opensourced;

#[opensourced]
pub fn selected_nonnull_as_ref_map_report(raw: &str) -> String {
    nonnull_as_ref_map_api::selected_nonnull_as_ref_map_report(raw)
}

pub fn dead_nonnull_as_ref_map_report(raw: &str) -> String {
    nonnull_as_ref_map_api::dead_nonnull_as_ref_map_report(raw)
}
