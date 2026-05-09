use opensourced::opensourced;

#[opensourced]
pub fn selected_while_let_report(raw: &str) -> String {
    while_let_api::selected_while_let_report(raw)
}

pub fn dead_while_let_report(raw: &str) -> String {
    while_let_api::dead_while_let_report(raw)
}
