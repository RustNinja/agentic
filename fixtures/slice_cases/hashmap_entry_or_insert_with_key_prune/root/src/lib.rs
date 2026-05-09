use opensourced::opensourced;

#[opensourced]
pub fn selected_hashmap_entry_or_insert_with_key_report(raw: &str) -> String {
    hashmap_entry_or_insert_with_key_api::selected_hashmap_entry_or_insert_with_key_report(raw)
}

pub fn dead_hashmap_entry_or_insert_with_key_report(raw: &str) -> String {
    hashmap_entry_or_insert_with_key_api::dead_hashmap_entry_or_insert_with_key_report(raw)
}
