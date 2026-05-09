pub fn selected_slice_split_at_mut_tail_iter_report(raw: &str) -> String {
    slice_split_at_mut_tail_iter_model::selected_slice_split_at_mut_tail_iter(raw)
}

pub fn dead_live_slice_split_at_mut_tail_iter_report(raw: &str) -> String {
    format!("dead-live-slice-split-at-mut-tail-iter:{raw}")
}
