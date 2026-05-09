use opensourced::opensourced;

#[opensourced]
pub fn selected_hashmap_get_key_value_map_report(raw: &str) -> String {
    hashmap_get_key_value_map_api::selected_hashmap_get_key_value_map_report(raw)
}

pub fn dead_hashmap_get_key_value_map_report(raw: &str) -> String {
    hashmap_get_key_value_map_api::dead_hashmap_get_key_value_map_report(raw)
}
