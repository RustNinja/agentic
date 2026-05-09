pub fn selected_vec_as_slice_iter_report(raw: &str) -> String {
    vec_as_slice_iter_model::selected_vec_as_slice_iter(raw)
}

pub fn dead_live_vec_as_slice_iter_report(raw: &str) -> String {
    format!("dead-vec-as-slice-iter-live-report:{raw}")
}
