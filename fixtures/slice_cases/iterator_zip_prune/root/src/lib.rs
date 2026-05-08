use opensourced::opensourced;

#[opensourced]
pub fn selected_zip_report(raw: &str) -> String {
    zip_api::selected_zip_report(raw)
}

pub fn dead_zip_report(raw: &str) -> String {
    zip_api::dead_zip_report(raw)
}
