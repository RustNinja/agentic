use opensourced::opensourced;

#[opensourced]
pub fn selected_slice_iter_copied_map_report(raw: &str) -> String {
    slice_iter_copied_map_api::selected_slice_iter_copied_map_report(raw)
}

pub fn dead_slice_iter_copied_map_report(raw: &str) -> String {
    slice_iter_copied_map_api::dead_slice_iter_copied_map_report(raw)
}
