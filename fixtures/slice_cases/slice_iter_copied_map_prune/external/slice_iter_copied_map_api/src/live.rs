pub fn selected_slice_iter_copied_map_report(raw: &str) -> String {
    slice_iter_copied_map_model::selected_slice_iter_copied_map(raw)
}

pub fn dead_live_slice_iter_copied_map_report(raw: &str) -> String {
    format!("dead-live-slice-iter-copied-map:{raw}")
}
