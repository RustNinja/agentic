mod live;

pub use live::selected_hashmap_remove_report;

pub fn dead_hashmap_remove_report(raw: &str) -> String {
    format!("dead-hashmap-remove-report:{raw}")
}
