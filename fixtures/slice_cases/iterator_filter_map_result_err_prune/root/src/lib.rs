use opensourced::opensourced;

#[opensourced]
pub fn selected_iterator_filter_map_result_err_report(raw: &str) -> String {
    iterator_filter_map_result_err_api::selected_iterator_filter_map_result_err_report(raw)
}

pub fn dead_iterator_filter_map_result_err_report(raw: &str) -> String {
    iterator_filter_map_result_err_api::dead_iterator_filter_map_result_err_report(raw)
}
