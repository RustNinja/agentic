use opensourced::opensourced;

#[opensourced]
pub fn selected_iterator_peekable_next_if_report(raw: &str) -> String {
    iterator_peekable_next_if_api::selected_iterator_peekable_next_if_report(raw)
}

pub fn dead_iterator_peekable_next_if_report(raw: &str) -> String {
    iterator_peekable_next_if_api::dead_iterator_peekable_next_if_report(raw)
}
