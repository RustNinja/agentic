mod live;

pub use live::selected_binaryheap_into_iter_report;

pub fn dead_binaryheap_into_iter_report(raw: &str) -> String {
    format!("dead-binaryheap-into-iter-report:{raw}")
}
