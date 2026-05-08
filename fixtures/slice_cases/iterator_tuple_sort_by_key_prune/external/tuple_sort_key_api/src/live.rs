pub fn selected_tuple_sort_by_key_report(raw: &str) -> String {
    tuple_sort_key_model::selected_tuple_sort_by_key(raw)
}

pub fn dead_live_tuple_sort_by_key_report(raw: &str) -> String {
    format!("dead-live-tuple-sort-by-key:{raw}")
}
