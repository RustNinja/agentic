use opensourced::opensourced;

#[opensourced]
pub fn selected_hashmap_extend_values_map_report(raw: &str) -> String {
    hashmap_extend_values_map_api::selected_hashmap_extend_values_map_report(raw)
}

pub fn dead_hashmap_extend_values_map_report(raw: &str) -> String {
    hashmap_extend_values_map_api::dead_hashmap_extend_values_map_report(raw)
}
