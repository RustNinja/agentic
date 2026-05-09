use opensourced::opensourced;

#[opensourced]
pub fn selected_collect_vecdeque_report(raw: &str) -> String {
    collect_vecdeque_api::selected_collect_vecdeque_report(raw)
}

pub fn dead_collect_vecdeque_report(raw: &str) -> String {
    collect_vecdeque_api::dead_collect_vecdeque_report(raw)
}
