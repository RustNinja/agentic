pub fn selected_mutex_into_inner_unwrap_map_report(raw: &str) -> String {
    mutex_into_inner_unwrap_map_model::selected_mutex_into_inner_unwrap_map(raw)
}

pub fn dead_live_mutex_into_inner_unwrap_map_report(raw: &str) -> String {
    format!("dead-live-mutex-into-inner-unwrap-map-report:{raw}")
}
