use opensourced::opensourced;

#[opensourced]
pub fn selected_const_report(raw: &str) -> String {
    const_api::selected_const_report(raw)
}

pub fn dead_const_report(raw: &str) -> String {
    const_api::dead_const_report(raw)
}
