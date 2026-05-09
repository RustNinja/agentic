mod live;

pub use live::selected_btreemap_iter_mut_pairs_report;

pub fn dead_btreemap_iter_mut_pairs_report(raw: &str) -> String {
    format!("dead-btreemap-iter-mut-pairs-report:{raw}")
}
