use opensourced::opensourced;

#[opensourced]
pub fn selected_btreemap_entry_or_insert_with_report(raw: &str) -> String {
    btreemap_entry_or_insert_with_api::selected_btreemap_entry_or_insert_with_report(raw)
}

pub fn dead_btreemap_entry_or_insert_with_report(raw: &str) -> String {
    btreemap_entry_or_insert_with_api::dead_btreemap_entry_or_insert_with_report(raw)
}
