pub fn selected_vec_resize_with_last_report(raw: &str) -> String {
    vec_resize_with_last_model::selected_vec_resize_with_last(raw)
}

pub fn dead_live_vec_resize_with_last_report(raw: &str) -> String {
    format!("dead-live-vec-resize-with-last:{raw}")
}
