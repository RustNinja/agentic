pub fn selected_str_rmatch_indices_map_report(raw: &str) -> String {
    str_rmatch_indices_map_model::selected_str_rmatch_indices_map(raw)
}

pub fn dead_live_str_rmatch_indices_map_report(raw: &str) -> String {
    format!("dead-str-rmatch-indices-map-live-report:{raw}")
}
