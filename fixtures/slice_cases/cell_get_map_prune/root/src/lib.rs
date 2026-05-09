use opensourced::opensourced;

#[opensourced]
pub fn selected_cell_get_map_report(raw: &str) -> String {
    cell_get_api::selected_cell_get_map_report(raw)
}

pub fn dead_cell_get_map_report(raw: &str) -> String {
    cell_get_api::dead_cell_get_map_report(raw)
}
