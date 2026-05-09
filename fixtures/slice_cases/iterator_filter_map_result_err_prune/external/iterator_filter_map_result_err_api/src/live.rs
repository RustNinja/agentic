pub fn selected_iterator_filter_map_result_err_report(raw: &str) -> String {
    iterator_filter_map_result_err_model::selected_iterator_filter_map_result_err(raw)
}

pub fn dead_live_iterator_filter_map_result_err_report(raw: &str) -> String {
    format!("dead-iterator-filter-map-result-err-live-report:{raw}")
}
