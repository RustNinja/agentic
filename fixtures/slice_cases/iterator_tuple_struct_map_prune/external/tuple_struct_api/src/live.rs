pub fn selected_tuple_struct_report(raw: &str) -> String {
    tuple_struct_model::selected_tuple_struct(raw)
}

pub fn dead_live_tuple_struct_report(raw: &str) -> String {
    format!("dead-live-tuple-struct:{raw}")
}
