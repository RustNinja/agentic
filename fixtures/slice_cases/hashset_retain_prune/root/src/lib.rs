use opensourced::opensourced;

#[opensourced]
pub fn selected_hashset_retain_report(raw: &str) -> String {
    hashset_retain_api::selected_hashset_retain_report(raw)
}

pub fn dead_hashset_retain_report(raw: &str) -> String {
    hashset_retain_api::dead_hashset_retain_report(raw)
}
