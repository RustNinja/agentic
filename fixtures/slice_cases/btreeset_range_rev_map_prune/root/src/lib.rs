use opensourced::opensourced;

#[opensourced]
pub fn selected_btreeset_range_rev_map_report(raw: &str) -> String {
    btreeset_range_rev_map_api::selected_btreeset_range_rev_map_report(raw)
}

pub fn dead_btreeset_range_rev_map_report(raw: &str) -> String {
    btreeset_range_rev_map_api::dead_btreeset_range_rev_map_report(raw)
}
