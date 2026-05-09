mod live;

pub use live::selected_hashset_get_report;

pub fn dead_hashset_get_report(raw: &str) -> String {
    format!("dead-hashset-get-report:{raw}")
}
