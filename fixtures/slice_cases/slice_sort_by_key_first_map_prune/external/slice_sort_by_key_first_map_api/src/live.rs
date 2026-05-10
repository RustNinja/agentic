pub fn selected_slice_sort_by_key_first_map_report(raw: &str) -> String {
    slice_sort_by_key_first_map_model::selected_slice_sort_by_key_first_map(raw)
}

pub fn dead_live_slice_sort_by_key_first_map_report(raw: &str) -> String {
    format!("dead-slice-sort-by-key-first-map-live-report:{raw}")
}
