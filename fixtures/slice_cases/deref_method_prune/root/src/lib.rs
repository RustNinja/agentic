use opensourced::opensourced;

#[opensourced]
pub fn selected_deref_report(raw: &str) -> String {
    deref_api::selected_deref_report(raw)
}

pub fn dead_deref_report(raw: &str) -> String {
    deref_api::dead_deref_report(raw)
}
