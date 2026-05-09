pub fn selected_iterator_filter_map_result_ok_report(raw: &str) -> String {
    iterator_filter_map_result_ok_model::selected_iterator_filter_map_result_ok(raw)
}

pub fn dead_live_iterator_filter_map_result_ok_report(raw: &str) -> String {
    format!("dead-iterator-filter-map-result-ok-live-report:{raw}")
}
