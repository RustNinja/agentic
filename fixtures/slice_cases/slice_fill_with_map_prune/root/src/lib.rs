use opensourced::opensourced;

#[opensourced]
pub fn selected_slice_fill_with_map_report(raw: &str) -> String {
    slice_fill_with_map_api::selected_slice_fill_with_map_report(raw)
}

pub fn dead_slice_fill_with_map_report(raw: &str) -> String {
    slice_fill_with_map_api::dead_slice_fill_with_map_report(raw)
}
