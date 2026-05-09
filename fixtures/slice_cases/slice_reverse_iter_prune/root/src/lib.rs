use opensourced::opensourced;

#[opensourced]
pub fn selected_slice_reverse_iter_report(raw: &str) -> String {
    slice_reverse_iter_api::selected_slice_reverse_iter_report(raw)
}

pub fn dead_slice_reverse_iter_report(raw: &str) -> String {
    slice_reverse_iter_api::dead_slice_reverse_iter_report(raw)
}
