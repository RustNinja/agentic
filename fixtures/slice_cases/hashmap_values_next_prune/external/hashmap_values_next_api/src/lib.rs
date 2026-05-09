mod live;

pub use live::selected_hashmap_values_next_report;

pub fn dead_hashmap_values_next_report(raw: &str) -> String {
    format!("dead-hashmap-values-next-report:{raw}")
}
