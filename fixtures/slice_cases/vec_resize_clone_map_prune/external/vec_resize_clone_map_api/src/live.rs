pub fn selected_vec_resize_clone_map_report(raw: &str) -> String {
    vec_resize_clone_map_model::selected_vec_resize_clone_map(raw)
}

pub fn dead_live_vec_resize_clone_map_report(raw: &str) -> String {
    format!("dead-vec-resize-clone-map-live-report:{raw}")
}
