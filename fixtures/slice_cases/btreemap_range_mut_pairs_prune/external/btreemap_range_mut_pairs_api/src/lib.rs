mod live;

pub use live::selected_btreemap_range_mut_pairs_report;

pub fn dead_btreemap_range_mut_pairs_report(raw: &str) -> String {
    format!("dead-btreemap-range-mut-pairs-report:{raw}")
}
