use opensourced::opensourced;

#[opensourced]
pub fn selected_option_ref_report(raw: &str) -> String {
    option_ref_api::selected_option_ref_report(raw)
}

pub fn dead_option_ref_report(raw: &str) -> String {
    option_ref_api::dead_option_ref_report(raw)
}
