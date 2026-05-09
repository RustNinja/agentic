use opensourced::opensourced;

#[opensourced]
pub fn selected_slice_windows_filter_map_report(raw: &str) -> String {
    slice_windows_filter_map_api::selected_slice_windows_filter_map_report(raw)
}

pub fn dead_slice_windows_filter_map_report(raw: &str) -> String {
    slice_windows_filter_map_api::dead_slice_windows_filter_map_report(raw)
}
