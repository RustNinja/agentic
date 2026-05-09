use opensourced::opensourced;

#[opensourced]
pub fn selected_btreemap_range_pairs_report(raw: &str) -> String {
    btreemap_range_pairs_api::selected_btreemap_range_pairs_report(raw)
}

pub fn dead_btreemap_range_pairs_report(raw: &str) -> String {
    btreemap_range_pairs_api::dead_btreemap_range_pairs_report(raw)
}
