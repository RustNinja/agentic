use opensourced::opensourced;

#[opensourced]
pub fn selected_cell_replace_map_report(raw: &str) -> String {
    cell_replace_map_api::selected_cell_replace_map_report(raw)
}

pub fn dead_cell_replace_map_report(raw: &str) -> String {
    cell_replace_map_api::dead_cell_replace_map_report(raw)
}
