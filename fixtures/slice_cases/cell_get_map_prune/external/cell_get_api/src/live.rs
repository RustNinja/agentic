pub fn selected_cell_get_map_report(raw: &str) -> String {
    cell_get_model::selected_cell_get_map(raw)
}

pub fn dead_live_cell_get_map_report(raw: &str) -> String {
    format!("dead-live-cell-get-map:{raw}")
}
