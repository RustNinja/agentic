mod live;

pub use live::selected_option_flatten_report;

pub fn dead_option_flatten_report(raw: &str) -> String {
    format!("dead-option-flatten-report:{raw}")
}
