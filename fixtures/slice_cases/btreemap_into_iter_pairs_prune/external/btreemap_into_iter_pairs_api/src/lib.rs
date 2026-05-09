mod live;

pub use live::selected_btreemap_into_iter_pairs_report;

pub fn dead_btreemap_into_iter_pairs_report(raw: &str) -> String {
    format!("dead-btreemap-into-iter-pairs-report:{raw}")
}
