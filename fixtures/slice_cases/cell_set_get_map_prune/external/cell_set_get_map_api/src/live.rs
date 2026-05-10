pub fn selected_cell_set_get_map_report(raw: &str) -> String {
    cell_set_get_map_model::selected_cell_set_get_map(raw)
}

pub fn dead_live_cell_set_get_map_report(raw: &str) -> String {
    format!("dead-cell-set-get-map-live-report:{raw}")
}
