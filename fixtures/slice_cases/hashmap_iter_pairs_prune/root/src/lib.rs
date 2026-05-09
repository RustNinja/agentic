use opensourced::opensourced;

#[opensourced]
pub fn selected_hashmap_iter_pairs_report(raw: &str) -> String {
    hashmap_iter_pairs_api::selected_hashmap_iter_pairs_report(raw)
}

pub fn dead_hashmap_iter_pairs_report(raw: &str) -> String {
    hashmap_iter_pairs_api::dead_hashmap_iter_pairs_report(raw)
}
