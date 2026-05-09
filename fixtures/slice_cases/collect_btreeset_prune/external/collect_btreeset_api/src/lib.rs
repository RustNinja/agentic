mod live;

pub use live::selected_collect_btreeset_report;

pub fn dead_collect_btreeset_report(raw: &str) -> String {
    format!("dead-collect-btreeset-report:{raw}")
}
