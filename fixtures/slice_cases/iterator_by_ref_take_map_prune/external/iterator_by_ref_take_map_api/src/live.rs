pub fn selected_iterator_by_ref_take_map_report(raw: &str) -> String {
    iterator_by_ref_take_map_model::selected_iterator_by_ref_take_map(raw)
}

pub fn dead_live_iterator_by_ref_take_map_report(raw: &str) -> String {
    format!("dead-live-iterator-by-ref-take-map:{raw}")
}
