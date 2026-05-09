mod live;

pub use live::selected_btreeset_take_report;

pub fn dead_btreeset_take_report(raw: &str) -> String {
    format!("dead-btreeset-take-report:{raw}")
}
