pub fn selected_const_report(raw: &str) -> String {
    const_support::selected_const(raw)
}

pub fn dead_live_const_report(raw: &str) -> String {
    format!("dead-live-const:{raw}")
}
