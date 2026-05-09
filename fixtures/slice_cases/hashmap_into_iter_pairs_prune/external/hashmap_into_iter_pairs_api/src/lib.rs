mod live;

pub use live::selected_hashmap_into_iter_pairs_report;

pub fn dead_hashmap_into_iter_pairs_report(raw: &str) -> String {
    format!("dead-hashmap-into-iter-pairs-report:{raw}")
}
