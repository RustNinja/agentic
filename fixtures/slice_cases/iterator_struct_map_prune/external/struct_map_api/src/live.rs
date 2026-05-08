pub fn selected_struct_map_report(raw: &str) -> String {
    struct_map_model::selected_struct_map(raw)
}

pub fn dead_live_struct_map_report(raw: &str) -> String {
    format!("dead-live-struct-map:{raw}")
}
