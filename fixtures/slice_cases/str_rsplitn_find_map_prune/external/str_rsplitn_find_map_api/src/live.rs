pub fn selected_str_rsplitn_find_map_report(raw: &str) -> String {
    str_rsplitn_find_map_model::selected_str_rsplitn_find_map(raw)
}

pub fn dead_live_str_rsplitn_find_map_report(raw: &str) -> String {
    format!("dead-str-rsplitn-find-map-live-report:{raw}")
}
