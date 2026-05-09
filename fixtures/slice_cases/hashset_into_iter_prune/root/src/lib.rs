use opensourced::opensourced;

#[opensourced]
pub fn selected_hashset_into_iter_report(raw: &str) -> String {
    hashset_into_iter_api::selected_hashset_into_iter_report(raw)
}

pub fn dead_hashset_into_iter_report(raw: &str) -> String {
    hashset_into_iter_api::dead_hashset_into_iter_report(raw)
}
