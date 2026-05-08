use opensourced::opensourced;

#[opensourced]
pub fn selected_struct_map_report(raw: &str) -> String {
    struct_map_api::selected_struct_map_report(raw)
}

pub fn dead_struct_map_report(raw: &str) -> String {
    struct_map_api::dead_struct_map_report(raw)
}
