mod live;

pub use live::selected_binaryheap_iter_find_report;

pub fn dead_binaryheap_iter_find_report(raw: &str) -> String {
    format!("dead-binaryheap-iter-find-report:{raw}")
}
