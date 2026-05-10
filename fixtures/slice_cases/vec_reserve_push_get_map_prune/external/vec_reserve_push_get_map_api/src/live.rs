pub fn selected_vec_reserve_push_get_map_report(raw: &str) -> String {
    vec_reserve_push_get_map_model::selected_vec_reserve_push_get_map(raw)
}

pub fn dead_live_vec_reserve_push_get_map_report(raw: &str) -> String {
    format!("dead-vec-reserve-push-get-map-live-report:{raw}")
}
