use opensourced::opensourced;

#[opensourced]
pub fn selected_hashmap_entry_or_default_report(raw: &str) -> String {
    hashmap_entry_or_default_api::selected_hashmap_entry_or_default_report(raw)
}

pub fn dead_hashmap_entry_or_default_report(raw: &str) -> String {
    hashmap_entry_or_default_api::dead_hashmap_entry_or_default_report(raw)
}
