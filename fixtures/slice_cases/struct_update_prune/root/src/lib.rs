use opensourced::opensourced;

#[opensourced]
pub fn selected_settings_report(raw: &str) -> String {
    settings_api::selected_settings_report(raw)
}

pub fn dead_settings_report(raw: &str) -> String {
    settings_api::dead_settings_report(raw)
}
