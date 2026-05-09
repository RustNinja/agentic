pub fn selected_slice_binary_search_by_key_report(raw: &str) -> String {
    slice_binary_search_by_key_model::selected_slice_binary_search_by_key(raw)
}

pub fn dead_live_slice_binary_search_by_key_report(raw: &str) -> String {
    format!("dead-slice-binary-search-by-key-live-report:{raw}")
}
