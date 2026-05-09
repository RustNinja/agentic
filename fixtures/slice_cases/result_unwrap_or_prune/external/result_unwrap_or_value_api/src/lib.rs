mod live;

pub use live::selected_result_unwrap_or_value_report;

pub fn dead_result_unwrap_or_value_report(raw: &str) -> String {
    format!("dead-result-unwrap-or-value-report:{raw}")
}
