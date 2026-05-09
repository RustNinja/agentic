use opensourced::opensourced;

#[opensourced]
pub fn selected_collect_hashmap_report(raw: &str) -> String {
    collect_hashmap_api::selected_collect_hashmap_report(raw)
}

pub fn dead_collect_hashmap_report(raw: &str) -> String {
    collect_hashmap_api::dead_collect_hashmap_report(raw)
}
