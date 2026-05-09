use opensourced::opensourced;

#[opensourced]
pub fn selected_vecdeque_as_mut_slices_iter_report(raw: &str) -> String {
    vecdeque_as_mut_slices_iter_api::selected_vecdeque_as_mut_slices_iter_report(raw)
}

pub fn dead_vecdeque_as_mut_slices_iter_report(raw: &str) -> String {
    vecdeque_as_mut_slices_iter_api::dead_vecdeque_as_mut_slices_iter_report(raw)
}
