mod live;

pub use live::selected_option_unwrap_direct_report;

pub fn dead_option_unwrap_direct_report(raw: &str) -> String {
    format!("dead-option-unwrap-direct-report:{raw}")
}
