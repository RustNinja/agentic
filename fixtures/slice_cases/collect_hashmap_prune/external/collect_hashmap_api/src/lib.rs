mod live;

pub use live::selected_collect_hashmap_report;

pub fn dead_collect_hashmap_report(raw: &str) -> String {
    format!("dead-collect-hashmap-report:{raw}")
}
