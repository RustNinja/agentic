pub fn selected_slice_mut_if_let_pattern_report(raw: &str) -> String {
    slice_mut_if_let_pattern_model::selected_slice_mut_if_let_pattern(raw)
}

pub fn dead_live_slice_mut_if_let_pattern_report(raw: &str) -> String {
    format!("dead-live-slice-mut-if-let-pattern-report:{raw}")
}
