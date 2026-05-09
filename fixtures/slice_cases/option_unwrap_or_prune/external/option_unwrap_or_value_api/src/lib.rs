mod live;

pub use live::selected_option_unwrap_or_value_report;

pub fn dead_option_unwrap_or_value_report(raw: &str) -> String {
    format!("dead-option-unwrap-or-value-report:{raw}")
}
