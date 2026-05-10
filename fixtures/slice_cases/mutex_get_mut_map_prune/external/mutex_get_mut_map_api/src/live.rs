pub fn selected_mutex_get_mut_map_report(raw: &str) -> String {
    mutex_get_mut_map_model::selected_mutex_get_mut_map(raw)
}

pub fn dead_live_mutex_get_mut_map_report(raw: &str) -> String {
    format!("dead-mutex-get-mut-map-live-report:{raw}")
}
