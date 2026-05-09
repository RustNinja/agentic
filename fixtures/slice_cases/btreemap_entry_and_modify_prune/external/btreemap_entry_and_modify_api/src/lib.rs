mod live;

pub use live::selected_btreemap_entry_and_modify_report;

pub fn dead_btreemap_entry_and_modify_report(raw: &str) -> String {
    format!("dead-btreemap-entry-and-modify-report:{raw}")
}
