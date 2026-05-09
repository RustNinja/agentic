pub fn selected_slice_swap_iter_report(raw: &str) -> String {
    slice_swap_iter_model::selected_slice_swap_iter(raw)
}

pub fn dead_live_slice_swap_iter_report(raw: &str) -> String {
    format!("dead-live-slice-swap-iter:{raw}")
}
