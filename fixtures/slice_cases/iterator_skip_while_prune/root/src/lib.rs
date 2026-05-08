use opensourced::opensourced;

#[opensourced]
pub fn selected_skip_while_report(raw: &str) -> String {
    skip_api::selected_skip_while_report(raw)
}

pub fn dead_skip_while_report(raw: &str) -> String {
    skip_api::dead_skip_while_report(raw)
}
