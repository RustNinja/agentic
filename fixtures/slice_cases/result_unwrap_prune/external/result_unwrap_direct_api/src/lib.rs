mod live;

pub use live::selected_result_unwrap_direct_report;

pub fn dead_result_unwrap_direct_report(raw: &str) -> String {
    format!("dead-result-unwrap-direct-report:{raw}")
}
