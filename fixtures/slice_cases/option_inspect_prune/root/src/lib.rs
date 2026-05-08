use opensourced::opensourced;

#[opensourced]
pub fn selected_option_inspect_report(raw: &str) -> String {
    option_inspect_api::selected_option_inspect_report(raw)
}

pub fn dead_option_inspect_report(raw: &str) -> String {
    option_inspect_api::dead_option_inspect_report(raw)
}
