mod live;

pub use live::selected_hashmap_entry_or_default_report;

pub fn dead_hashmap_entry_or_default_report(raw: &str) -> String {
    format!("dead-hashmap-entry-or-default-report:{raw}")
}
