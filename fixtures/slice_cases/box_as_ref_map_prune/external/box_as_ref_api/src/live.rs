pub fn selected_box_as_ref_map_report(raw: &str) -> String {
    box_as_ref_model::selected_box_as_ref_map(raw)
}

pub fn dead_live_box_as_ref_map_report(raw: &str) -> String {
    format!("dead-live-box-as-ref-map:{raw}")
}
