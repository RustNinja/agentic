use opensourced::opensourced;

#[opensourced]
pub fn selected_hashmap_entry_remove_map_report(raw: &str) -> String {
    hashmap_entry_remove_map_api::selected_hashmap_entry_remove_map_report(raw)
}

pub fn dead_hashmap_entry_remove_map_report(raw: &str) -> String {
    hashmap_entry_remove_map_api::dead_hashmap_entry_remove_map_report(raw)
}
