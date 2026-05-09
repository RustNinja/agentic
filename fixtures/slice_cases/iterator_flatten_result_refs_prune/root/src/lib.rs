use opensourced::opensourced;

#[opensourced]
pub fn selected_iterator_flatten_result_refs_report(raw: &str) -> String {
    iterator_flatten_result_refs_api::selected_iterator_flatten_result_refs_report(raw)
}

pub fn dead_iterator_flatten_result_refs_report(raw: &str) -> String {
    iterator_flatten_result_refs_api::dead_iterator_flatten_result_refs_report(raw)
}
