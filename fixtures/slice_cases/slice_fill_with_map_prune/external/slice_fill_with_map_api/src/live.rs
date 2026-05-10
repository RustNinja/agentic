pub fn selected_slice_fill_with_map_report(raw: &str) -> String {
    slice_fill_with_map_model::selected_slice_fill_with_map(raw)
}

pub fn dead_live_slice_fill_with_map_report(raw: &str) -> String {
    format!("dead-slice-fill-with-map-live-report:{raw}")
}
