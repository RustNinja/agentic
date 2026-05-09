mod live;

pub use live::selected_hashmap_entry_and_modify_report;

pub fn dead_hashmap_entry_and_modify_report(raw: &str) -> String {
    format!("dead-hashmap-entry-and-modify-report:{raw}")
}
