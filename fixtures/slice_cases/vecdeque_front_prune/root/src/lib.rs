use opensourced::opensourced;

#[opensourced]
pub fn selected_vecdeque_front_report(raw: &str) -> String {
    vecdeque_front_api::selected_vecdeque_front_report(raw)
}

pub fn dead_vecdeque_front_report(raw: &str) -> String {
    vecdeque_front_api::dead_vecdeque_front_report(raw)
}
