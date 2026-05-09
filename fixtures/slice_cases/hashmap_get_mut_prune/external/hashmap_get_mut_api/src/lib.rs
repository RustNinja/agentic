mod live;

pub use live::selected_hashmap_get_mut_report;

pub fn dead_hashmap_get_mut_report(raw: &str) -> String {
    format!("dead-hashmap-get-mut-report:{raw}")
}
