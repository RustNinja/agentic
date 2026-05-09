mod live;

pub use live::selected_btreeset_range_find_report;

pub fn dead_btreeset_range_find_report(raw: &str) -> String {
    format!("dead-btreeset-range-find-report:{raw}")
}
