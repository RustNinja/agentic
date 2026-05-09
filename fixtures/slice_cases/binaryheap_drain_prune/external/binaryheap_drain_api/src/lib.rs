mod live;

pub use live::selected_binaryheap_drain_report;

pub fn dead_binaryheap_drain_report(raw: &str) -> String {
    format!("dead-binaryheap-drain-report:{raw}")
}
