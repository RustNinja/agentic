mod live;

pub use live::selected_option_expect_report;

pub fn dead_option_expect_report(raw: &str) -> String {
    format!("dead-option-expect-report:{raw}")
}
