pub fn selected_cycle_nth_report(raw: &str) -> String {
    cycle_nth_model::selected_cycle_nth(raw)
}

pub fn dead_live_cycle_nth_report(raw: &str) -> String {
    format!("dead-cycle-nth-live-report:{raw}")
}
