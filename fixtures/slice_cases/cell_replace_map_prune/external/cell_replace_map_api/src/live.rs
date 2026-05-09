pub fn selected_cell_replace_map_report(raw: &str) -> String {
    cell_replace_map_model::selected_cell_replace_map(raw)
}

pub fn dead_live_cell_replace_map_report(raw: &str) -> String {
    format!("dead-live-cell-replace-map:{raw}")
}
