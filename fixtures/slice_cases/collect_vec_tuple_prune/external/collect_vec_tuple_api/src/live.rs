pub fn selected_collect_vec_tuple_report(raw: &str) -> String {
    collect_vec_tuple_model::selected_collect_vec_tuple(raw)
}

pub fn dead_live_collect_vec_tuple_report(raw: &str) -> String {
    format!("dead-collect-vec-tuple-live-report:{raw}")
}
