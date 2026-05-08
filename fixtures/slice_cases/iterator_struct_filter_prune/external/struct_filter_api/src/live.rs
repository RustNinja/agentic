pub fn selected_struct_filter_report(raw: &str) -> String {
    struct_filter_model::selected_struct_filter(raw)
}

pub fn dead_live_struct_filter_report(raw: &str) -> String {
    format!("dead-live-struct-filter:{raw}")
}
