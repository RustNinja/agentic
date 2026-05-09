use opensourced::opensourced;

#[opensourced]
pub fn selected_vecdeque_pop_back_report(raw: &str) -> String {
    vecdeque_pop_back_api::selected_vecdeque_pop_back_report(raw)
}

pub fn dead_vecdeque_pop_back_report(raw: &str) -> String {
    vecdeque_pop_back_api::dead_vecdeque_pop_back_report(raw)
}
