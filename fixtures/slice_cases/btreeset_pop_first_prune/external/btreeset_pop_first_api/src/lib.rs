mod live;

pub use live::selected_btreeset_pop_first_report;

pub fn dead_btreeset_pop_first_report(raw: &str) -> String {
    format!("dead-btreeset-pop-first-report:{raw}")
}
