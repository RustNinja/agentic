use opensourced::opensourced;

#[opensourced]
pub fn selected_vecdeque_append_iter_report(raw: &str) -> String {
    vecdeque_append_iter_api::selected_vecdeque_append_iter_report(raw)
}

pub fn dead_vecdeque_append_iter_report(raw: &str) -> String {
    vecdeque_append_iter_api::dead_vecdeque_append_iter_report(raw)
}
