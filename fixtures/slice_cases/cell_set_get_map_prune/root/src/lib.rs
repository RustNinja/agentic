use opensourced::opensourced;

#[opensourced]
pub fn selected_cell_set_get_map_report(raw: &str) -> String {
    cell_set_get_map_api::selected_cell_set_get_map_report(raw)
}

pub fn dead_cell_set_get_map_report(raw: &str) -> String {
    cell_set_get_map_api::dead_cell_set_get_map_report(raw)
}
