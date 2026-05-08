use opensourced::opensourced;

#[opensourced]
pub fn selected_iterator_report(raw: &str) -> String {
    iterator_api::selected_iterator_report(raw)
}

pub fn dead_iterator_report(raw: &str) -> String {
    iterator_api::dead_iterator_report(raw)
}
