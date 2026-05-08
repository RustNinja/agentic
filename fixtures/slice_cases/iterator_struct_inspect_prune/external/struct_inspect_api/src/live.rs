pub fn selected_struct_inspect_report(raw: &str) -> String {
    struct_inspect_model::selected_struct_inspect(raw)
}

pub fn dead_live_struct_inspect_report(raw: &str) -> String {
    format!("dead-live-struct-inspect:{raw}")
}
