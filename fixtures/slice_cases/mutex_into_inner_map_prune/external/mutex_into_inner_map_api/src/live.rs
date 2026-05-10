pub fn selected_mutex_into_inner_map_report(raw: &str) -> String {
    mutex_into_inner_map_model::selected_mutex_into_inner_map(raw)
}

pub fn dead_live_mutex_into_inner_map_report(raw: &str) -> String {
    format!("dead-live-mutex-into-inner-map-report:{raw}")
}
