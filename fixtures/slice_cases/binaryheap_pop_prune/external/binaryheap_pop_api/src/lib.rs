mod live;

pub use live::selected_binaryheap_pop_report;

pub fn dead_binaryheap_pop_report(raw: &str) -> String {
    format!("dead-binaryheap-pop-report:{raw}")
}
