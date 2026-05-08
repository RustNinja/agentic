use opensourced::opensourced;

#[opensourced]
pub fn selected_facade_report(raw: &str) -> String {
    facade_api::selected_facade_report(raw)
}

pub fn dead_facade_report(raw: &str) -> String {
    facade_api::dead_facade_report(raw)
}
