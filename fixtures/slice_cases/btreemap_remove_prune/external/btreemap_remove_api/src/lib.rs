mod live;

pub use live::selected_btreemap_remove_report;

pub fn dead_btreemap_remove_report(raw: &str) -> String {
    format!("dead-btreemap-remove-report:{raw}")
}
