use opensourced::opensourced;

#[opensourced]
pub fn selected_display_report(raw: &str) -> String {
    display_api::selected_display_report(raw)
}

pub fn dead_display_report(raw: &str) -> String {
    display_api::dead_display_report(raw)
}
