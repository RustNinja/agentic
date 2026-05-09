use opensourced::opensourced;

#[opensourced]
pub fn selected_vecdeque_retain_report(raw: &str) -> String {
    vecdeque_retain_api::selected_vecdeque_retain_report(raw)
}

pub fn dead_vecdeque_retain_report(raw: &str) -> String {
    vecdeque_retain_api::dead_vecdeque_retain_report(raw)
}
