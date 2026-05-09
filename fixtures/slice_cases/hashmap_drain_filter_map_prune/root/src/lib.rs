use opensourced::opensourced;

#[opensourced]
pub fn selected_hashmap_drain_filter_map_report(raw: &str) -> String {
    hashmap_drain_filter_map_api::selected_hashmap_drain_filter_map_report(raw)
}

pub fn dead_hashmap_drain_filter_map_report(raw: &str) -> String {
    hashmap_drain_filter_map_api::dead_hashmap_drain_filter_map_report(raw)
}
