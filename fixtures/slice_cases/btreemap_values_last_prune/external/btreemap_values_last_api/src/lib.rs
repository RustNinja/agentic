mod live;

pub use live::selected_btreemap_values_last_report;

pub fn dead_btreemap_values_last_report(raw: &str) -> String {
    format!("dead-btreemap-values-last-report:{raw}")
}
