use opensourced::opensourced;

#[opensourced]
pub fn selected_vecdeque_iter_find_report(raw: &str) -> String {
    vecdeque_iter_find_api::selected_vecdeque_iter_find_report(raw)
}

pub fn dead_vecdeque_iter_find_report(raw: &str) -> String {
    vecdeque_iter_find_api::dead_vecdeque_iter_find_report(raw)
}
