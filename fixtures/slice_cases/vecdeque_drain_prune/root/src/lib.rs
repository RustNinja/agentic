use opensourced::opensourced;

#[opensourced]
pub fn selected_vecdeque_drain_report(raw: &str) -> String {
    vecdeque_drain_api::selected_vecdeque_drain_report(raw)
}

pub fn dead_vecdeque_drain_report(raw: &str) -> String {
    vecdeque_drain_api::dead_vecdeque_drain_report(raw)
}
