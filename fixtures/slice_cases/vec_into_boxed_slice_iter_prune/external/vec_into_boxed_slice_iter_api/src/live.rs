pub fn selected_vec_into_boxed_slice_iter_report(raw: &str) -> String {
    vec_into_boxed_slice_iter_model::selected_vec_into_boxed_slice_iter(raw)
}

pub fn dead_live_vec_into_boxed_slice_iter_report(raw: &str) -> String {
    format!("dead-live-vec-into-boxed-slice-iter:{raw}")
}
