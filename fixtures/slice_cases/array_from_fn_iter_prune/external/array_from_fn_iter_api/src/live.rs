pub fn selected_array_from_fn_iter_report(raw: &str) -> String {
    array_from_fn_iter_model::selected_array_from_fn_iter(raw)
}

pub fn dead_live_array_from_fn_iter_report(raw: &str) -> String {
    format!("dead-live-array-from-fn-iter:{raw}")
}
