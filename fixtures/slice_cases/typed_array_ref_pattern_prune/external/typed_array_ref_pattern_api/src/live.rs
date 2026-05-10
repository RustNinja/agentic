pub fn selected_typed_array_ref_pattern_report(raw: &str) -> String {
    typed_array_ref_pattern_model::selected_typed_array_ref_pattern(raw)
}

pub fn dead_live_typed_array_ref_pattern_report(raw: &str) -> String {
    format!("dead-live-typed-array-ref-pattern-report:{raw}")
}
