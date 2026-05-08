pub fn selected_sort_by_report(raw: &str) -> String {
    sort_model::selected_sort_by(raw)
}

pub fn dead_live_sort_by_report(raw: &str) -> String {
    format!("dead-live-sort:{raw}")
}
