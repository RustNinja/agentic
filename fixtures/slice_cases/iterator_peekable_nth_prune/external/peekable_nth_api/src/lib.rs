mod live;

pub use live::selected_peekable_nth_report;

pub fn dead_peekable_nth_report(raw: &str) -> String {
    format!("dead-peekable-nth-report:{raw}")
}
