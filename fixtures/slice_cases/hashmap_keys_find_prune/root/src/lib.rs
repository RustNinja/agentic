use opensourced::opensourced;

#[opensourced]
pub fn selected_hashmap_keys_find_report(raw: &str) -> String {
    hashmap_keys_find_api::selected_hashmap_keys_find_report(raw)
}

pub fn dead_hashmap_keys_find_report(raw: &str) -> String {
    hashmap_keys_find_api::dead_hashmap_keys_find_report(raw)
}
