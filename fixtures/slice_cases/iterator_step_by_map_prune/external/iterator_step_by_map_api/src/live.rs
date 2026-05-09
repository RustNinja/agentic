pub fn selected_iterator_step_by_map_report(raw: &str) -> String {
    iterator_step_by_map_model::selected_iterator_step_by_map(raw)
}

pub fn dead_live_iterator_step_by_map_report(raw: &str) -> String {
    format!("dead-live-iterator-step-by-map:{raw}")
}
