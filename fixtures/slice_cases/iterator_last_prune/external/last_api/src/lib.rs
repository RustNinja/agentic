mod live;

pub use live::selected_last_report;

pub fn dead_last_report(raw: &str) -> String {
    format!("dead-last-report:{raw}")
}
