mod live;

pub use live::selected_hashmap_retain_report;

pub fn dead_hashmap_retain_report(raw: &str) -> String {
    format!("dead-hashmap-retain-report:{raw}")
}
