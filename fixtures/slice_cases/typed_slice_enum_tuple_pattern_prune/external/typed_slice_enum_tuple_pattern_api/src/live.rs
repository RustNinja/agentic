pub fn selected_typed_slice_enum_tuple_pattern_report(raw: &str) -> String {
    typed_slice_enum_tuple_pattern_model::selected_typed_slice_enum_tuple_pattern(raw)
}

pub fn dead_live_typed_slice_enum_tuple_pattern_report(raw: &str) -> String {
    format!("dead-live-typed-slice-enum-tuple-pattern-report:{raw}")
}
