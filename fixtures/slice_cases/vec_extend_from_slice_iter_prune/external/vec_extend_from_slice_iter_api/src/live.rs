pub fn selected_vec_extend_from_slice_iter_report(raw: &str) -> String {
    vec_extend_from_slice_iter_model::selected_vec_extend_from_slice_iter(raw)
}

pub fn dead_live_vec_extend_from_slice_iter_report(raw: &str) -> String {
    format!("dead-live-vec-extend-from-slice-iter:{raw}")
}
