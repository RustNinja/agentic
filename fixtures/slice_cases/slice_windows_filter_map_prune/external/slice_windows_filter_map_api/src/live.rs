pub fn selected_slice_windows_filter_map_report(raw: &str) -> String {
    slice_windows_filter_map_model::selected_slice_windows_filter_map(raw)
}

pub fn dead_live_slice_windows_filter_map_report(raw: &str) -> String {
    format!("dead-slice-windows-filter-map-live-report:{raw}")
}
