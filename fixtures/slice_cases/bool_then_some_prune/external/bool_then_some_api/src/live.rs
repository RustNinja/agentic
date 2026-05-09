pub fn selected_bool_then_some_report(raw: &str) -> String {
    bool_then_some_model::selected_bool_then_some(raw)
}

pub fn dead_live_bool_then_some_report(raw: &str) -> String {
    format!("dead-bool-then-some-live-report:{raw}")
}
