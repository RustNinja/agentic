pub fn selected_option_ref_report(raw: &str) -> String {
    option_ref_model::selected_option_ref(raw)
}

pub fn dead_live_option_ref_report(raw: &str) -> String {
    format!("dead-live-option-ref:{raw}")
}
