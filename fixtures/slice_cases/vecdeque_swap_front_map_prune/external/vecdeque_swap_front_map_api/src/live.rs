pub fn selected_vecdeque_swap_front_map_report(raw: &str) -> String {
    vecdeque_swap_front_map_model::selected_vecdeque_swap_front_map(raw)
}

pub fn dead_live_vecdeque_swap_front_map_report(raw: &str) -> String {
    format!("dead-vecdeque-swap-front-map-live-report:{raw}")
}
