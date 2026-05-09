use opensourced::opensourced;

#[opensourced]
pub fn selected_slice_binary_search_by_report(raw: &str) -> String {
    slice_binary_search_by_api::selected_slice_binary_search_by_report(raw)
}

pub fn dead_slice_binary_search_by_report(raw: &str) -> String {
    slice_binary_search_by_api::dead_slice_binary_search_by_report(raw)
}
