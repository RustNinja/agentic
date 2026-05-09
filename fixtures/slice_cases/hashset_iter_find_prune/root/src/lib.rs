use opensourced::opensourced;

#[opensourced]
pub fn selected_hashset_iter_find_report(raw: &str) -> String {
    hashset_iter_find_api::selected_hashset_iter_find_report(raw)
}

pub fn dead_hashset_iter_find_report(raw: &str) -> String {
    hashset_iter_find_api::dead_hashset_iter_find_report(raw)
}
