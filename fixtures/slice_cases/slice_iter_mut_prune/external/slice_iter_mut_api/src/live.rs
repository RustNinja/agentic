pub fn selected_slice_iter_mut_report(raw: &str) -> String {
    slice_iter_mut_model::selected_slice_iter_mut(raw)
}

pub fn dead_live_slice_iter_mut_report(raw: &str) -> String {
    format!("dead-slice-iter-mut-live-report:{raw}")
}
