mod live;

pub use live::selected_rposition_report;

pub fn dead_rposition_report(raw: &str) -> String {
    format!("dead-rposition-report:{raw}")
}
