use opensourced::opensourced;

#[opensourced]
pub fn selected_hashmap_get_report(raw: &str) -> String {
    hashmap_get_api::selected_hashmap_get_report(raw)
}

pub fn dead_hashmap_get_report(raw: &str) -> String {
    hashmap_get_api::dead_hashmap_get_report(raw)
}
