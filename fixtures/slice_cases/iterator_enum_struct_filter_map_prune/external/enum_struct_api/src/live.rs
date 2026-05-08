pub fn selected_enum_struct_report(raw: &str) -> String {
    enum_struct_model::selected_enum_struct(raw)
}

pub fn dead_live_enum_struct_report(raw: &str) -> String {
    format!("dead-live-enum-struct:{raw}")
}
