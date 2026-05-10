use opensourced::opensourced;

#[opensourced]
pub fn selected_iterator_rfind_report(raw: &str) -> String {
    iterator_rfind_api::selected_iterator_rfind_report(raw)
}

pub fn dead_iterator_rfind_report(raw: &str) -> String {
    iterator_rfind_api::dead_iterator_rfind_report(raw)
}
