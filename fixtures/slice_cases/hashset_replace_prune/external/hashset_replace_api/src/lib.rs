mod live;

pub use live::selected_hashset_replace_report;

pub fn dead_hashset_replace_report(raw: &str) -> String {
    format!("dead-hashset-replace-report:{raw}")
}
