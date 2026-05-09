mod live;

pub use live::selected_result_expect_report;

pub fn dead_result_expect_report(raw: &str) -> String {
    format!("dead-result-expect-report:{raw}")
}
