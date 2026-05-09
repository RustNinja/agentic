mod live;

pub use live::selected_btreeset_pop_last_report;

pub fn dead_btreeset_pop_last_report(raw: &str) -> String {
    format!("dead-btreeset-pop-last-report:{raw}")
}
