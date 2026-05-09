pub fn selected_vec_insert_get_map_report(raw: &str) -> String {
    vec_insert_get_map_model::selected_vec_insert_get_map(raw)
}

pub fn dead_live_vec_insert_get_map_report(raw: &str) -> String {
    format!("dead-vec-insert-get-map-live-report:{raw}")
}
