pub fn selected_option_tuple_struct_report(raw: &str) -> String {
    option_tuple_struct_model::selected_option_tuple_struct(raw)
}

pub fn dead_live_option_tuple_struct_report(raw: &str) -> String {
    format!("dead-live-option-tuple-struct:{raw}")
}
