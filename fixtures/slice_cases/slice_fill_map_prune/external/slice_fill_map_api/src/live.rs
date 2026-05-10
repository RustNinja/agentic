pub fn selected_slice_fill_map_report(raw: &str) -> String {
    slice_fill_map_model::selected_slice_fill_map(raw)
}

pub fn dead_live_slice_fill_map_report(raw: &str) -> String {
    format!("dead-slice-fill-map-live-report:{raw}")
}
