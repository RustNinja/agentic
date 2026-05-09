mod live;

pub use live::selected_option_xor_report;

pub fn dead_option_xor_report(raw: &str) -> String {
    format!("dead-option-xor-report:{raw}")
}
