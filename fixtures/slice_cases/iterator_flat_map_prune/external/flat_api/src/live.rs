pub fn selected_flat_map_report(raw: &str) -> String {
    flat_model::selected_flat_map(raw)
}

pub fn dead_live_flat_map_report(raw: &str) -> String {
    format!("dead-live:{raw}")
}
