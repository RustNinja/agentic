use opensourced::opensourced;

#[opensourced]
pub fn selected_hashset_get_report(raw: &str) -> String {
    hashset_get_api::selected_hashset_get_report(raw)
}

pub fn dead_hashset_get_report(raw: &str) -> String {
    hashset_get_api::dead_hashset_get_report(raw)
}
