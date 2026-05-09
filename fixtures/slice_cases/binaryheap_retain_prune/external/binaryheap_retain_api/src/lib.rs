mod live;

pub use live::selected_binaryheap_retain_report;

pub fn dead_binaryheap_retain_report(raw: &str) -> String {
    format!("dead-binaryheap-retain-report:{raw}")
}
