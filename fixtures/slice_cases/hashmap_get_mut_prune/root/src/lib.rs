use opensourced::opensourced;

#[opensourced]
pub fn selected_hashmap_get_mut_report(raw: &str) -> String {
    hashmap_get_mut_api::selected_hashmap_get_mut_report(raw)
}

pub fn dead_hashmap_get_mut_report(raw: &str) -> String {
    hashmap_get_mut_api::dead_hashmap_get_mut_report(raw)
}
