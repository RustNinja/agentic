pub fn selected_typed_slice_named_struct_pattern_report(raw: &str) -> String {
    typed_slice_named_struct_pattern_model::selected_typed_slice_named_struct_pattern(raw)
}

pub fn dead_live_typed_slice_named_struct_pattern_report(raw: &str) -> String {
    format!("dead-live-typed-slice-named-struct-pattern-report:{raw}")
}
