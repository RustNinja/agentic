pub fn selected_tuple_for_each_report(raw: &str) -> String {
    tuple_for_each_model::selected_tuple_for_each(raw)
}

pub fn dead_live_tuple_for_each_report(raw: &str) -> String {
    format!("dead-live-tuple-for-each:{raw}")
}
