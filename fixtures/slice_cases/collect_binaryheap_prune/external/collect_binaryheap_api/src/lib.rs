mod live;

pub use live::selected_collect_binaryheap_report;

pub fn dead_collect_binaryheap_report(raw: &str) -> String {
    format!("dead-collect-binaryheap-report:{raw}")
}
