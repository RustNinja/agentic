mod live;

pub use live::selected_hashset_retain_report;

pub fn dead_hashset_retain_report(raw: &str) -> String {
    format!("dead-hashset-retain-report:{raw}")
}
