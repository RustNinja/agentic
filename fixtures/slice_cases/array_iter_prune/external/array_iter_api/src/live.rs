pub fn selected_array_iter_report(raw: &str) -> String {
    array_iter_model::selected_array_iter(raw)
}

pub fn dead_live_array_iter_report(raw: &str) -> String {
    format!("dead-array-iter-live-report:{raw}")
}
