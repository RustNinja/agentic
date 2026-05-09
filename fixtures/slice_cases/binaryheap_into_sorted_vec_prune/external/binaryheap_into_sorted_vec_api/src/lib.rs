mod live;

pub use live::selected_binaryheap_into_sorted_vec_report;

pub fn dead_binaryheap_into_sorted_vec_report(raw: &str) -> String {
    format!("dead-binaryheap-into-sorted-vec-report:{raw}")
}
