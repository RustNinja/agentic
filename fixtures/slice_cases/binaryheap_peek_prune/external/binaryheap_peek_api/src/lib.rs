mod live;

pub use live::selected_binaryheap_peek_report;

pub fn dead_binaryheap_peek_report(raw: &str) -> String {
    format!("dead-binaryheap-peek-report:{raw}")
}
