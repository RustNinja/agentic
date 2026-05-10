pub fn selected_slice_let_else_pattern_report(raw: &str) -> String {
    slice_let_else_pattern_model::selected_slice_let_else_pattern(raw)
}

pub fn dead_live_slice_let_else_pattern_report(raw: &str) -> String {
    format!("dead-live-slice-let-else-pattern-report:{raw}")
}
