pub fn selected_pin_into_inner_map_report(raw: &str) -> String {
    pin_into_inner_map_model::selected_pin_into_inner_map(raw)
}

pub fn dead_live_pin_into_inner_map_report(raw: &str) -> String {
    format!("dead-pin-into-inner-map-live-report:{raw}")
}
