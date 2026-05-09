pub fn selected_btreemap_entry_and_modify_report(raw: &str) -> String {
    btreemap_entry_and_modify_model::selected_btreemap_entry_and_modify(raw)
}

pub fn dead_live_btreemap_entry_and_modify_report(raw: &str) -> String {
    format!("dead-btreemap-entry-and-modify-live-report:{raw}")
}
