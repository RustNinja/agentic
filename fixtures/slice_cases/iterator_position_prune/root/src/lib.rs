use opensourced::opensourced;

#[opensourced]
pub fn selected_iterator_position_report(raw: &str) -> String {
    iterator_position_api::selected_iterator_position_report(raw)
}

pub fn dead_iterator_position_report(raw: &str) -> String {
    iterator_position_api::dead_iterator_position_report(raw)
}
