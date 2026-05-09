mod live;

pub use live::selected_hashmap_get_report;

pub fn dead_hashmap_get_report(raw: &str) -> String {
    format!("dead-hashmap-get-report:{raw}")
}
