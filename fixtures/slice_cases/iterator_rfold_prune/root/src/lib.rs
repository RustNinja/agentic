use opensourced::opensourced;

#[opensourced]
pub fn selected_iterator_rfold_report(raw: &str) -> String {
    iterator_rfold_api::selected_iterator_rfold_report(raw)
}

pub fn dead_iterator_rfold_report(raw: &str) -> String {
    iterator_rfold_api::dead_iterator_rfold_report(raw)
}
