pub fn selected_array_into_iter_skip_map_report(raw: &str) -> String {
    array_into_iter_skip_map_model::selected_array_into_iter_skip_map(raw)
}

pub fn dead_live_array_into_iter_skip_map_report(raw: &str) -> String {
    format!("dead-live-array-into-iter-skip-map:{raw}")
}
