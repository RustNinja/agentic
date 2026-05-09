mod live;

pub use live::selected_slice_binary_search_by_report;

pub fn dead_slice_binary_search_by_report(raw: &str) -> String {
    format!("dead-slice-binary-search-by-report:{raw}")
}
