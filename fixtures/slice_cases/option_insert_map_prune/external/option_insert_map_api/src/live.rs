pub fn selected_option_insert_map_report(raw: &str) -> String {
    option_insert_map_model::selected_option_insert_map(raw)
}

pub fn dead_live_option_insert_map_report(raw: &str) -> String {
    format!("dead-live-option-insert-map:{raw}")
}
