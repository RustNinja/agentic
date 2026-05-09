use opensourced::opensourced;

#[opensourced]
pub fn selected_hashmap_entry_and_modify_report(raw: &str) -> String {
    hashmap_entry_and_modify_api::selected_hashmap_entry_and_modify_report(raw)
}

pub fn dead_hashmap_entry_and_modify_report(raw: &str) -> String {
    hashmap_entry_and_modify_api::dead_hashmap_entry_and_modify_report(raw)
}
