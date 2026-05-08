use opensourced::opensourced;

#[opensourced]
pub fn selected_flat_map_report(raw: &str) -> String {
    flat_api::selected_flat_map_report(raw)
}

pub fn dead_flat_map_report(raw: &str) -> String {
    flat_api::dead_flat_map_report(raw)
}
