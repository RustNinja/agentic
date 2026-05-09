use opensourced::opensourced;

#[opensourced]
pub fn selected_hashset_replace_report(raw: &str) -> String {
    hashset_replace_api::selected_hashset_replace_report(raw)
}

pub fn dead_hashset_replace_report(raw: &str) -> String {
    hashset_replace_api::dead_hashset_replace_report(raw)
}
