pub fn selected_option_struct_report(raw: &str) -> String {
    option_struct_model::selected_option_struct(raw)
}

pub fn dead_live_option_struct_report(raw: &str) -> String {
    format!("dead-live-option-struct:{raw}")
}
