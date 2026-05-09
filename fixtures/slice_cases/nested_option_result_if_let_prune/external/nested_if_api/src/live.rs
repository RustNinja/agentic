pub fn selected_nested_if_report(raw: &str) -> String {
    nested_if_model::selected_nested_if(raw)
}

pub fn dead_live_nested_if_report(raw: &str) -> String {
    format!("dead-live-nested-if:{raw}")
}
