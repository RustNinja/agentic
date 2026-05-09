pub fn selected_pin_box_as_mut_map_report(raw: &str) -> String {
    pin_box_as_mut_model::selected_pin_box_as_mut_map(raw)
}

pub fn dead_live_pin_box_as_mut_map_report(raw: &str) -> String {
    format!("dead-live-pin-box-as-mut-map:{raw}")
}
