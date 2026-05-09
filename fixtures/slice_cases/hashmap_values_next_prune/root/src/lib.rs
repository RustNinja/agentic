use opensourced::opensourced;

#[opensourced]
pub fn selected_hashmap_values_next_report(raw: &str) -> String {
    hashmap_values_next_api::selected_hashmap_values_next_report(raw)
}

pub fn dead_hashmap_values_next_report(raw: &str) -> String {
    hashmap_values_next_api::dead_hashmap_values_next_report(raw)
}
