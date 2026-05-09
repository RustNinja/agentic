mod live;

pub use live::selected_min_by_report;

pub fn dead_min_by_report(raw: &str) -> String {
    format!("dead-min-by-report:{raw}")
}
