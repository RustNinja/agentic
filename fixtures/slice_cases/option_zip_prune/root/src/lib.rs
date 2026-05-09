use opensourced::opensourced;

#[opensourced]
pub fn selected_option_zip_report(raw: &str) -> String {
    option_zip_api::selected_option_zip_report(raw)
}

pub fn dead_option_zip_report(raw: &str) -> String {
    option_zip_api::dead_option_zip_report(raw)
}
