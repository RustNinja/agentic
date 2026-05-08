use opensourced::opensourced;

#[opensourced]
pub fn selected_entry_map_report(raw: &str) -> String {
    entry_map_api::selected_entry_map_report(raw)
}

pub fn dead_entry_map_report(raw: &str) -> String {
    entry_map_api::dead_entry_map_report(raw)
}
