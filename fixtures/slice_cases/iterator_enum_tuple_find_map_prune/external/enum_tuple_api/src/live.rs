pub fn selected_enum_tuple_report(raw: &str) -> String {
    enum_tuple_model::selected_enum_tuple(raw)
}

pub fn dead_live_enum_tuple_report(raw: &str) -> String {
    format!("dead-live-enum-tuple:{raw}")
}
