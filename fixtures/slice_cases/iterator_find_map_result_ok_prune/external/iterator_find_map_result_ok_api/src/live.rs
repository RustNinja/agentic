pub fn selected_iterator_find_map_result_ok_report(raw: &str) -> String {
    iterator_find_map_result_ok_model::selected_iterator_find_map_result_ok(raw)
}

pub fn dead_live_iterator_find_map_result_ok_report(raw: &str) -> String {
    format!("dead-iterator-find-map-result-ok-live-report:{raw}")
}
