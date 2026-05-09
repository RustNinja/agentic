pub fn selected_slice_rotate_right_iter_report(raw: &str) -> String {
    slice_rotate_right_iter_model::selected_slice_rotate_right_iter(raw)
}

pub fn dead_live_slice_rotate_right_iter_report(raw: &str) -> String {
    format!("dead-live-slice-rotate-right-iter:{raw}")
}
