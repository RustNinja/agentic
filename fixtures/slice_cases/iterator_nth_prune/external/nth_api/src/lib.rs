mod live;

pub use live::selected_nth_report;

pub fn dead_nth_report(raw: &str) -> String {
    format!("dead-nth-report:{raw}")
}
