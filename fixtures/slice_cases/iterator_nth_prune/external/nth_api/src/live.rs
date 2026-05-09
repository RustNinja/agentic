pub fn selected_nth_report(raw: &str) -> String {
    nth_model::selected_nth(raw)
}

pub fn dead_live_nth_report(raw: &str) -> String {
    format!("dead-nth-live-report:{raw}")
}
