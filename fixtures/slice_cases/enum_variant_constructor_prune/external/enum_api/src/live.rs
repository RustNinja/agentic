pub fn selected_enum_report(raw: &str) -> String {
    enum_model::selected_enum(raw)
}

pub fn dead_live_enum_report(raw: &str) -> String {
    format!("dead-live-enum:{raw}")
}
