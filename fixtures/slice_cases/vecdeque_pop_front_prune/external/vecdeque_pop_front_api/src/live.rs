pub fn selected_vecdeque_pop_front_report(raw: &str) -> String {
    vecdeque_pop_front_model::selected_vecdeque_pop_front(raw)
}

pub fn dead_live_vecdeque_pop_front_report(raw: &str) -> String {
    format!("dead-vecdeque-pop-front-live-report:{raw}")
}
