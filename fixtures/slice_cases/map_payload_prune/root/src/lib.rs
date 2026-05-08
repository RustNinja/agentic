use opensourced::opensourced;

#[opensourced]
pub fn selected_map_report(raw: &str) -> String {
    map_api::selected_map_report(raw)
}

pub fn dead_map_report(raw: &str) -> String {
    map_api::dead_map_report(raw)
}
