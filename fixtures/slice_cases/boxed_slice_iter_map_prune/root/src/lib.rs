use opensourced::opensourced;

#[opensourced]
pub fn selected_boxed_slice_iter_map_report(raw: &str) -> String {
    boxed_slice_iter_map_api::selected_boxed_slice_iter_map_report(raw)
}

pub fn dead_boxed_slice_iter_map_report(raw: &str) -> String {
    boxed_slice_iter_map_api::dead_boxed_slice_iter_map_report(raw)
}
