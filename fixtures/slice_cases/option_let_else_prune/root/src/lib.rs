use opensourced::opensourced;

#[opensourced]
pub fn selected_option_let_report(raw: &str) -> String {
    option_let_api::selected_option_let_report(raw)
}

pub fn dead_option_let_report(raw: &str) -> String {
    option_let_api::dead_option_let_report(raw)
}
