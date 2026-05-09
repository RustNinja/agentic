pub fn selected_hashmap_entry_and_modify_report(raw: &str) -> String {
    hashmap_entry_and_modify_model::selected_hashmap_entry_and_modify(raw)
}

pub fn dead_live_hashmap_entry_and_modify_report(raw: &str) -> String {
    format!("dead-hashmap-entry-and-modify-live-report:{raw}")
}
