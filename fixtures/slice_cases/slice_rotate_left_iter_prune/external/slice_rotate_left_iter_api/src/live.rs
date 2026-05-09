pub fn selected_slice_rotate_left_iter_report(raw: &str) -> String {
    slice_rotate_left_iter_model::selected_slice_rotate_left_iter(raw)
}

pub fn dead_live_slice_rotate_left_iter_report(raw: &str) -> String {
    format!("dead-live-slice-rotate-left-iter:{raw}")
}
