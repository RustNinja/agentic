use opensourced::opensourced;

#[opensourced]
pub fn selected_enum_flat_map_report(raw: &str) -> String {
    enum_flat_map_api::selected_enum_flat_map_report(raw)
}

pub fn dead_enum_flat_map_report(raw: &str) -> String {
    enum_flat_map_api::dead_enum_flat_map_report(raw)
}
