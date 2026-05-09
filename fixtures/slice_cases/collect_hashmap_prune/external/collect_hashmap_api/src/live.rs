pub fn selected_collect_hashmap_report(raw: &str) -> String {
    collect_hashmap_model::selected_collect_hashmap(raw)
}

pub fn dead_live_collect_hashmap_report(raw: &str) -> String {
    format!("dead-collect-hashmap-live-report:{raw}")
}
