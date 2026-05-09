use opensourced::opensourced;

#[opensourced]
pub fn selected_btreemap_into_keys_map_report(raw: &str) -> String {
    btreemap_into_keys_map_api::selected_btreemap_into_keys_map_report(raw)
}

pub fn dead_btreemap_into_keys_map_report(raw: &str) -> String {
    btreemap_into_keys_map_api::dead_btreemap_into_keys_map_report(raw)
}
