use opensourced::opensourced;

#[opensourced]
pub fn selected_generic_report(raw: &str) -> String {
    generic_api::selected_generic_report(raw)
}

pub fn dead_generic_report(raw: &str) -> String {
    generic_api::dead_generic_report(raw)
}
