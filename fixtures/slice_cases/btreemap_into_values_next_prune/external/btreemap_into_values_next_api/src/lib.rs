mod live;

pub use live::selected_btreemap_into_values_next_report;

pub fn dead_btreemap_into_values_next_report(raw: &str) -> String {
    format!("dead-btreemap-into-values-next-report:{raw}")
}
