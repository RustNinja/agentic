pub fn selected_vec_as_mut_slice_iter_mut_report(raw: &str) -> String {
    vec_as_mut_slice_iter_mut_model::selected_vec_as_mut_slice_iter_mut(raw)
}

pub fn dead_live_vec_as_mut_slice_iter_mut_report(raw: &str) -> String {
    format!("dead-vec-as-mut-slice-iter-mut-live-report:{raw}")
}
