pub fn selected_box_pin_as_ref_map_report(raw: &str) -> String {
    box_pin_as_ref_map_model::selected_box_pin_as_ref_map(raw)
}

pub fn dead_live_box_pin_as_ref_map_report(raw: &str) -> String {
    format!("dead-live-box-pin-as-ref-map:{raw}")
}
