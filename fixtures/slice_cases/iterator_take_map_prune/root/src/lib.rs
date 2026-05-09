use opensourced::opensourced;

#[opensourced]
pub fn selected_iterator_take_map_report(raw: &str) -> String {
    iterator_take_map_api::selected_iterator_take_map_report(raw)
}

pub fn dead_iterator_take_map_report(raw: &str) -> String {
    iterator_take_map_api::dead_iterator_take_map_report(raw)
}
