mod live;

pub use live::selected_slice_binary_search_by_key_report;

pub fn dead_slice_binary_search_by_key_report(raw: &str) -> String {
    format!("dead-slice-binary-search-by-key-report:{raw}")
}
