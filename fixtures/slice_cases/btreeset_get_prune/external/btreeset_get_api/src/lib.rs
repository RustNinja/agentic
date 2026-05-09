mod live;

pub use live::selected_btreeset_get_report;

pub fn dead_btreeset_get_report(raw: &str) -> String {
    format!("dead-btreeset-get-report:{raw}")
}
