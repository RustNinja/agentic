pub fn selected_slice_strip_prefix_iter_report(raw: &str) -> String {
    slice_strip_prefix_iter_model::selected_slice_strip_prefix_iter(raw)
}

pub fn dead_live_slice_strip_prefix_iter_report(raw: &str) -> String {
    format!("dead-live-slice-strip-prefix-iter:{raw}")
}
