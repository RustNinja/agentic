pub fn selected_tuple_inspect_report(raw: &str) -> String {
    tuple_inspect_model::selected_tuple_inspect(raw)
}

pub fn dead_live_tuple_inspect_report(raw: &str) -> String {
    format!("dead-live-tuple-inspect:{raw}")
}
