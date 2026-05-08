use opensourced::opensourced;

#[opensourced]
pub fn selected_take_while_report(raw: &str) -> String {
    take_api::selected_take_while_report(raw)
}

pub fn dead_take_while_report(raw: &str) -> String {
    take_api::dead_take_while_report(raw)
}
