pub fn selected_box_leak_mut_map_report(raw: &str) -> String {
    box_leak_mut_map_model::selected_box_leak_mut_map(raw)
}

pub fn dead_live_box_leak_mut_map_report(raw: &str) -> String {
    format!("dead-box-leak-mut-map-live-report:{raw}")
}
