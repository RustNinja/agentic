pub fn selected_vecdeque_pop_back_report(raw: &str) -> String {
    vecdeque_pop_back_model::selected_vecdeque_pop_back(raw)
}

pub fn dead_live_vecdeque_pop_back_report(raw: &str) -> String {
    format!("dead-vecdeque-pop-back-live-report:{raw}")
}
