pub fn selected_tuple_filter_report(raw: &str) -> String {
    tuple_filter_model::selected_tuple_filter(raw)
}

pub fn dead_live_tuple_filter_report(raw: &str) -> String {
    format!("dead-live-tuple-filter:{raw}")
}
