use opensourced::opensourced;

#[opensourced]
pub fn selected_slice_iter_mut_report(raw: &str) -> String {
    slice_iter_mut_api::selected_slice_iter_mut_report(raw)
}

pub fn dead_slice_iter_mut_report(raw: &str) -> String {
    slice_iter_mut_api::dead_slice_iter_mut_report(raw)
}
