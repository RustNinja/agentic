pub fn selected_btreemap_entry_or_default_report(raw: &str) -> String {
    btreemap_entry_or_default_model::selected_btreemap_entry_or_default(raw)
}

pub fn dead_live_btreemap_entry_or_default_report(raw: &str) -> String {
    format!("dead-btreemap-entry-or-default-live-report:{raw}")
}
