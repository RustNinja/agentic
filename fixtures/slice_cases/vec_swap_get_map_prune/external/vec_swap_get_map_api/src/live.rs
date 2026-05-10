pub fn selected_vec_swap_get_map_report(raw: &str) -> String {
    vec_swap_get_map_model::selected_vec_swap_get_map(raw)
}

pub fn dead_live_vec_swap_get_map_report(raw: &str) -> String {
    format!("dead-vec-swap-get-map-live-report:{raw}")
}
