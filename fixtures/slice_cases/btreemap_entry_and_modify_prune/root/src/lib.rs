use opensourced::opensourced;

#[opensourced]
pub fn selected_btreemap_entry_and_modify_report(raw: &str) -> String {
    btreemap_entry_and_modify_api::selected_btreemap_entry_and_modify_report(raw)
}

pub fn dead_btreemap_entry_and_modify_report(raw: &str) -> String {
    btreemap_entry_and_modify_api::dead_btreemap_entry_and_modify_report(raw)
}
