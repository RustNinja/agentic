pub fn selected_option_enum_named_report(raw: &str) -> String {
    option_enum_named_model::selected_option_enum_named(raw)
}

pub fn dead_live_option_enum_named_report(raw: &str) -> String {
    format!("dead-live-option-enum-named:{raw}")
}
