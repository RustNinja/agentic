use opensourced::opensourced;

#[opensourced]
pub fn selected_btreemap_entry_or_default_report(raw: &str) -> String {
    btreemap_entry_or_default_api::selected_btreemap_entry_or_default_report(raw)
}

pub fn dead_btreemap_entry_or_default_report(raw: &str) -> String {
    btreemap_entry_or_default_api::dead_btreemap_entry_or_default_report(raw)
}
