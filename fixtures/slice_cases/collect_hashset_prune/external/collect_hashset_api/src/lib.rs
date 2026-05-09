mod live;

pub use live::selected_collect_hashset_report;

pub fn dead_collect_hashset_report(raw: &str) -> String {
    format!("dead-collect-hashset-report:{raw}")
}
