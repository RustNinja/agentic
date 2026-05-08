pub fn selected_enum_if_let_report(raw: &str) -> String {
    enum_if_let_model::selected_enum_if_let(raw)
}

pub fn dead_live_enum_if_let_report(raw: &str) -> String {
    format!("dead-live-enum-if-let:{raw}")
}
