pub fn selected_option_as_deref_mut_map_report(raw: &str) -> String {
    option_as_deref_mut_model::selected_option_as_deref_mut_map(raw)
}

pub fn dead_live_option_as_deref_mut_map_report(raw: &str) -> String {
    format!("dead-live-option-as-deref-mut-map:{raw}")
}
