mod live;

pub use live::selected_collect_btreemap_report;

pub fn dead_collect_btreemap_report(raw: &str) -> String {
    format!("dead-collect-btreemap-report:{raw}")
}
