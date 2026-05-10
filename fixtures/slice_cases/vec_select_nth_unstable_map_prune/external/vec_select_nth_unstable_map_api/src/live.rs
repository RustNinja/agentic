pub fn selected_vec_select_nth_unstable_map_report(raw: &str) -> String {
    vec_select_nth_unstable_map_model::selected_vec_select_nth_unstable_map(raw)
}

pub fn dead_live_vec_select_nth_unstable_map_report(raw: &str) -> String {
    format!("dead-vec-select-nth-unstable-map-live-report:{raw}")
}
