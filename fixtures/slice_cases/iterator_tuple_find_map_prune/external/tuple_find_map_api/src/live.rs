pub fn selected_tuple_find_map_report(raw: &str) -> String {
    tuple_find_map_model::selected_tuple_find_map(raw)
}

pub fn dead_live_tuple_find_map_report(raw: &str) -> String {
    format!("dead-live-tuple-find-map:{raw}")
}
