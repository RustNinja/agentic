use opensourced::opensourced;

#[opensourced]
pub fn selected_option_xor_report(raw: &str) -> String {
    option_xor_api::selected_option_xor_report(raw)
}

pub fn dead_option_xor_report(raw: &str) -> String {
    option_xor_api::dead_option_xor_report(raw)
}
