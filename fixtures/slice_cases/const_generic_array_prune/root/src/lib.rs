use opensourced::opensourced;

#[opensourced]
pub fn selected_array_report(raw: &str) -> String {
    array_api::selected_array_report(raw)
}

pub fn dead_array_report(raw: &str) -> String {
    array_api::dead_array_report(raw)
}
