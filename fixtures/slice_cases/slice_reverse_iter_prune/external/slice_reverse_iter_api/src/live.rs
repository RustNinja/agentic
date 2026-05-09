pub fn selected_slice_reverse_iter_report(raw: &str) -> String {
    slice_reverse_iter_model::selected_slice_reverse_iter(raw)
}

pub fn dead_live_slice_reverse_iter_report(raw: &str) -> String {
    format!("dead-live-slice-reverse-iter:{raw}")
}
