use opensourced::opensourced;

#[opensourced]
pub fn selected_array_iter_report(raw: &str) -> String {
    array_iter_api::selected_array_iter_report(raw)
}

pub fn dead_array_iter_report(raw: &str) -> String {
    array_iter_api::dead_array_iter_report(raw)
}
