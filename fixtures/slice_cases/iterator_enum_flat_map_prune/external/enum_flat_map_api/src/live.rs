pub fn selected_enum_flat_map_report(raw: &str) -> String {
    enum_flat_map_model::selected_enum_flat_map(raw)
}

pub fn dead_live_enum_flat_map_report(raw: &str) -> String {
    format!("dead-live-enum-flat-map:{raw}")
}
