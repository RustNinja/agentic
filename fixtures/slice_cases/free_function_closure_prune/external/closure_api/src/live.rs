pub fn selected_free_closure_report(raw: &str) -> String {
    closure_model::with_free_payload(raw, |payload| payload.render())
}

pub fn dead_live_free_closure_report(raw: &str) -> String {
    format!("dead-live-free-closure:{raw}")
}
