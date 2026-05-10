pub fn selected_str_replacen_map_report(raw: &str) -> String {
    str_replacen_map_model::selected_str_replacen_map(raw)
}

pub fn dead_live_str_replacen_map_report(raw: &str) -> String {
    format!("dead-str-replacen-map-live-report:{raw}")
}
