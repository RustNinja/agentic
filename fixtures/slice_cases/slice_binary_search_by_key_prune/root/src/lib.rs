use opensourced::opensourced;

#[opensourced]
pub fn selected_slice_binary_search_by_key_report(raw: &str) -> String {
    slice_binary_search_by_key_api::selected_slice_binary_search_by_key_report(raw)
}

pub fn dead_slice_binary_search_by_key_report(raw: &str) -> String {
    slice_binary_search_by_key_api::dead_slice_binary_search_by_key_report(raw)
}
