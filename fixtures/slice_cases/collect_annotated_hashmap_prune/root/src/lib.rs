use opensourced::opensourced;

#[opensourced]
pub fn selected_collect_annotated_hashmap_report(raw: &str) -> String {
    collect_annotated_hashmap_api::selected_collect_annotated_hashmap_report(raw)
}

pub fn dead_collect_annotated_hashmap_report(raw: &str) -> String {
    collect_annotated_hashmap_api::dead_live_collect_annotated_hashmap(raw)
}
