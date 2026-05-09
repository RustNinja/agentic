pub fn selected_option_get_or_insert_default_map_report(raw: &str) -> String {
    option_get_or_insert_default_map_model::selected_option_get_or_insert_default_map(raw)
}

pub fn dead_live_option_get_or_insert_default_map_report(raw: &str) -> String {
    format!("dead-live-option-get-or-insert-default-map:{raw}")
}
