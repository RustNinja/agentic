pub fn selected_box_into_raw_from_raw_map_report(raw: &str) -> String {
    box_into_raw_from_raw_map_model::selected_box_into_raw_from_raw_map(raw)
}

pub fn dead_live_box_into_raw_from_raw_map_report(raw: &str) -> String {
    format!("dead-live-box-into-raw-from-raw-map-report:{raw}")
}
