pub fn selected_skip_while_report(raw: &str) -> String {
    skip_model::selected_skip_while(raw)
}

pub fn dead_live_skip_while_report(raw: &str) -> String {
    format!("dead-live-skip:{raw}")
}
