mod live;

pub use live::selected_rev_last_report;

pub fn dead_rev_last_report(raw: &str) -> String {
    format!("dead-rev-last-report:{raw}")
}
