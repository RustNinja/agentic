pub fn selected_poll_report(raw: &str) -> String {
    poll_support::selected_poll(raw)
}

pub fn dead_live_poll_report(raw: &str) -> String {
    format!("dead-live-poll:{raw}")
}
