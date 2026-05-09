mod live;

pub use live::selected_btreemap_entry_or_default_report;

pub fn dead_btreemap_entry_or_default_report(raw: &str) -> String {
    format!("dead-btreemap-entry-or-default-report:{raw}")
}
