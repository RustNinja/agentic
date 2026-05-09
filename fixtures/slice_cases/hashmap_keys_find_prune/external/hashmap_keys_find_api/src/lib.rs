mod live;

pub use live::selected_hashmap_keys_find_report;

pub fn dead_hashmap_keys_find_report(raw: &str) -> String {
    format!("dead-hashmap-keys-find-report:{raw}")
}
