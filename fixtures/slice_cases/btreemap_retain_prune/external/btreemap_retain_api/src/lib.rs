mod live;

pub use live::selected_btreemap_retain_report;

pub fn dead_btreemap_retain_report(raw: &str) -> String {
    format!("dead-btreemap-retain-report:{raw}")
}
