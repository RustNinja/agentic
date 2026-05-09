mod live;

pub use live::selected_hashmap_iter_mut_pairs_report;

pub fn dead_hashmap_iter_mut_pairs_report(raw: &str) -> String {
    format!("dead-hashmap-iter-mut-pairs-report:{raw}")
}
