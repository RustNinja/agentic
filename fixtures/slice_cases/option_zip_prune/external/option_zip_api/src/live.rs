pub fn selected_option_zip_report(raw: &str) -> String {
    option_zip_model::selected_option_zip(raw)
}

pub fn dead_live_option_zip_report(raw: &str) -> String {
    format!("dead-option-zip-live-report:{raw}")
}
