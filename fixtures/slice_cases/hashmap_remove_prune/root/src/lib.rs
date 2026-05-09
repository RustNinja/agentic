use opensourced::opensourced;

#[opensourced]
pub fn selected_hashmap_remove_report(raw: &str) -> String {
    hashmap_remove_api::selected_hashmap_remove_report(raw)
}

pub fn dead_hashmap_remove_report(raw: &str) -> String {
    hashmap_remove_api::dead_hashmap_remove_report(raw)
}
