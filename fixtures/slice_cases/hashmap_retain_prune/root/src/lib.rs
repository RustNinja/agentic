use opensourced::opensourced;

#[opensourced]
pub fn selected_hashmap_retain_report(raw: &str) -> String {
    hashmap_retain_api::selected_hashmap_retain_report(raw)
}

pub fn dead_hashmap_retain_report(raw: &str) -> String {
    hashmap_retain_api::dead_hashmap_retain_report(raw)
}
