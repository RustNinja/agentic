pub fn selected_hashmap_entry_match_report(raw: &str) -> String {
    hashmap_entry_match_model::selected_hashmap_entry_match(raw)
}

pub fn dead_live_hashmap_entry_match_report(raw: &str) -> String {
    format!("dead-hashmap-entry-match-live-report:{raw}")
}
