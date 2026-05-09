pub fn selected_vec_split_off_into_iter_report(raw: &str) -> String {
    vec_split_off_into_iter_model::selected_vec_split_off_into_iter(raw)
}

pub fn dead_live_vec_split_off_into_iter_report(raw: &str) -> String {
    format!("dead-live-vec-split-off-into-iter:{raw}")
}
