use opensourced::opensourced;

#[opensourced]
pub fn selected_iterator_peekable_peek_mut_report(raw: &str) -> String {
    iterator_peekable_peek_mut_api::selected_iterator_peekable_peek_mut_report(raw)
}

pub fn dead_iterator_peekable_peek_mut_report(raw: &str) -> String {
    iterator_peekable_peek_mut_api::dead_iterator_peekable_peek_mut_report(raw)
}
