pub fn selected_slice_split_last_map_report(raw: &str) -> String {
    slice_split_last_map_model::selected_slice_split_last_map(raw)
}

pub fn dead_live_slice_split_last_map_report(raw: &str) -> String {
    format!("dead-live-slice-split-last-map:{raw}")
}
