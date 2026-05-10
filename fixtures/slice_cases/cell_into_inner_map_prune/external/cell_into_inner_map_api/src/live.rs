pub fn selected_cell_into_inner_map_report(raw: &str) -> String {
    cell_into_inner_map_model::selected_cell_into_inner_map(raw)
}

pub fn dead_live_cell_into_inner_map_report(raw: &str) -> String {
    format!("dead-live-cell-into-inner-map-report:{raw}")
}
