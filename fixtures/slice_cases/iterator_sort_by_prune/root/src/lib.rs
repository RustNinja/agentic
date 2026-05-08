use opensourced::opensourced;

#[opensourced]
pub fn selected_sort_by_report(raw: &str) -> String {
    sort_api::selected_sort_by_report(raw)
}

pub fn dead_sort_by_report(raw: &str) -> String {
    sort_api::dead_sort_by_report(raw)
}
