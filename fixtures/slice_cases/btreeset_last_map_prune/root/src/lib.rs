use opensourced::opensourced;

#[opensourced]
pub fn selected_btreeset_last_map_report(raw: &str) -> String {
    btreeset_last_map_api::selected_btreeset_last_map_report(raw)
}

pub fn dead_btreeset_last_map_report(raw: &str) -> String {
    btreeset_last_map_api::dead_btreeset_last_map_report(raw)
}
