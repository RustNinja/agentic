use opensourced::opensourced;

#[opensourced]
pub fn selected_find_report(raw: &str) -> String {
    find_api::selected_find_report(raw)
}

pub fn dead_find_report(raw: &str) -> String {
    find_api::dead_find_report(raw)
}
