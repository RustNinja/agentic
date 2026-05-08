pub fn selected_tuple_map_report(raw: &str) -> String {
    tuple_map_model::selected_tuple_map(raw)
}

pub fn dead_live_tuple_map_report(raw: &str) -> String {
    format!("dead-live-tuple-map:{raw}")
}
