pub fn selected_slice_binary_search_by_report(raw: &str) -> String {
    slice_binary_search_by_model::selected_slice_binary_search_by(raw)
}

pub fn dead_live_slice_binary_search_by_report(raw: &str) -> String {
    format!("dead-slice-binary-search-by-live-report:{raw}")
}
