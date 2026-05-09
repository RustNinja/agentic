pub fn selected_str_splitn_find_map_report(raw: &str) -> String {
    str_splitn_find_map_model::selected_str_splitn_find_map(raw)
}

pub fn dead_live_str_splitn_find_map_report(raw: &str) -> String {
    format!("dead-str-splitn-find-map-live-report:{raw}")
}
