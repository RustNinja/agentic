pub fn selected_nested_struct_report(raw: &str) -> String {
    nested_struct_model::selected_nested_struct(raw)
}

pub fn dead_live_nested_struct_report(raw: &str) -> String {
    format!("dead-live-nested-struct:{raw}")
}
