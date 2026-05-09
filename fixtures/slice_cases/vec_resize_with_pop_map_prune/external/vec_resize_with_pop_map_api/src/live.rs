pub fn selected_vec_resize_with_pop_map_report(raw: &str) -> String {
    vec_resize_with_pop_map_model::selected_vec_resize_with_pop_map(raw)
}

pub fn dead_live_vec_resize_with_pop_map_report(raw: &str) -> String {
    format!("dead-vec-resize-with-pop-map-live-report:{raw}")
}
