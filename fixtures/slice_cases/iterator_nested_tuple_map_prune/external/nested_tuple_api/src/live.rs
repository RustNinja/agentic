pub fn selected_nested_tuple_map_report(raw: &str) -> String {
    nested_tuple_model::selected_nested_tuple_map(raw)
}

pub fn dead_live_nested_tuple_map_report(raw: &str) -> String {
    format!("dead-live-nested-tuple-map:{raw}")
}
