pub fn selected_map_report(raw: &str) -> String {
    map_model::selected_map(raw)
}

pub fn dead_live_map_report(raw: &str) -> String {
    format!("dead-live-map:{raw}")
}
