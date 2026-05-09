mod live;

pub use live::selected_btreeset_retain_report;

pub fn dead_btreeset_retain_report(raw: &str) -> String {
    format!("dead-btreeset-retain-report:{raw}")
}
