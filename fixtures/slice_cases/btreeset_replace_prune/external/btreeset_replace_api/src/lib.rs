mod live;

pub use live::selected_btreeset_replace_report;

pub fn dead_btreeset_replace_report(raw: &str) -> String {
    format!("dead-btreeset-replace-report:{raw}")
}
