mod live;

pub use live::selected_btreemap_range_pairs_report;

pub fn dead_btreemap_range_pairs_report(raw: &str) -> String {
    format!("dead-btreemap-range-pairs-report:{raw}")
}
