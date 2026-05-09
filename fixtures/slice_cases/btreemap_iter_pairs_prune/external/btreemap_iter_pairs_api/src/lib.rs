mod live;

pub use live::selected_btreemap_iter_pairs_report;

pub fn dead_btreemap_iter_pairs_report(raw: &str) -> String {
    format!("dead-btreemap-iter-pairs-report:{raw}")
}
