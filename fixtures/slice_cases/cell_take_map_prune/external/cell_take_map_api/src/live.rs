pub fn selected_cell_take_map_report(raw: &str) -> String {
    cell_take_map_model::selected_cell_take_map(raw)
}

pub fn dead_live_cell_take_map_report(raw: &str) -> String {
    format!("dead-live-cell-take-map-report:{raw}")
}
