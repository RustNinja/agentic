use opensourced::opensourced;

#[opensourced]
pub fn selected_vecdeque_split_off_into_iter_report(raw: &str) -> String {
    vecdeque_split_off_into_iter_api::selected_vecdeque_split_off_into_iter_report(raw)
}

pub fn dead_vecdeque_split_off_into_iter_report(raw: &str) -> String {
    vecdeque_split_off_into_iter_api::dead_vecdeque_split_off_into_iter_report(raw)
}
