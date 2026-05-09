use opensourced::opensourced;

#[opensourced]
pub fn selected_hashmap_remove_entry_map_report(raw: &str) -> String {
    hashmap_remove_entry_map_api::selected_hashmap_remove_entry_map_report(raw)
}

pub fn dead_hashmap_remove_entry_map_report(raw: &str) -> String {
    hashmap_remove_entry_map_api::dead_hashmap_remove_entry_map_report(raw)
}
