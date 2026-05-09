mod live;

pub use live::selected_hashmap_into_values_next_report;

pub fn dead_hashmap_into_values_next_report(raw: &str) -> String {
    format!("dead-hashmap-into-values-next-report:{raw}")
}
