pub fn selected_slice_splitn_mut_filter_map_report(raw: &str) -> String {
    slice_splitn_mut_filter_map_model::selected_slice_splitn_mut_filter_map(raw)
}

pub fn dead_live_slice_splitn_mut_filter_map_report(raw: &str) -> String {
    format!("dead-slice-splitn-mut-filter-map-live-report:{raw}")
}
