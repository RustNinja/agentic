mod live;

pub use live::selected_btreemap_get_report;

pub fn dead_btreemap_get_report(raw: &str) -> String {
    format!("dead-btreemap-get-report:{raw}")
}
