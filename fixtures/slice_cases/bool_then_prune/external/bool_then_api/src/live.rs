pub fn selected_bool_then_report(raw: &str) -> String {
    bool_then_model::selected_bool_then(raw)
}

pub fn dead_live_bool_then_report(raw: &str) -> String {
    format!("dead-bool-then-live-report:{raw}")
}
