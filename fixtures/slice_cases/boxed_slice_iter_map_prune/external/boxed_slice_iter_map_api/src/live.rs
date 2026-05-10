pub fn selected_boxed_slice_iter_map_report(raw: &str) -> String {
    boxed_slice_iter_map_model::selected_boxed_slice_iter_map(raw)
}

pub fn dead_live_boxed_slice_iter_map_report(raw: &str) -> String {
    format!("dead-live-boxed-slice-iter-map-report:{raw}")
}
