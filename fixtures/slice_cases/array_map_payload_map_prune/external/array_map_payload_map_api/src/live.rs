pub fn selected_array_map_payload_map_report(raw: &str) -> String {
    array_map_payload_map_model::selected_array_map_payload_map(raw)
}

pub fn dead_live_array_map_payload_map_report(raw: &str) -> String {
    format!("dead-array-map-payload-map-live-report:{raw}")
}
