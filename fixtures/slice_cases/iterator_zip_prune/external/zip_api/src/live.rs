pub fn selected_zip_report(raw: &str) -> String {
    zip_model::selected_zip(raw)
}

pub fn dead_live_zip_report(raw: &str) -> String {
    format!("dead-live-zip:{raw}")
}
