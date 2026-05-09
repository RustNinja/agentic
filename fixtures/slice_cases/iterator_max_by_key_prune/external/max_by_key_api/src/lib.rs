mod live;

pub use live::selected_max_by_key_report;

pub fn dead_max_by_key_report(raw: &str) -> String {
    format!("dead-max-by-key-report:{raw}")
}
