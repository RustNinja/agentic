pub fn selected_pin_box_as_ref_map_report(raw: &str) -> String {
    pin_box_as_ref_model::selected_pin_box_as_ref_map(raw)
}

pub fn dead_live_pin_box_as_ref_map_report(raw: &str) -> String {
    format!("dead-live-pin-box-as-ref-map:{raw}")
}
