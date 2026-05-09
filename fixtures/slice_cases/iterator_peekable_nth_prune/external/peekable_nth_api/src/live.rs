pub fn selected_peekable_nth_report(raw: &str) -> String {
    peekable_nth_model::selected_peekable_nth(raw)
}

pub fn dead_live_peekable_nth_report(raw: &str) -> String {
    format!("dead-peekable-nth-live-report:{raw}")
}
