pub fn selected_closure_report(raw: &str) -> String {
    closure_model::selected_closure(raw)
}

pub fn dead_live_closure_report(raw: &str) -> String {
    format!("dead-live-closure:{raw}")
}
