use opensourced::opensourced;

#[opensourced]
pub fn selected_hashset_drain_report(raw: &str) -> String {
    hashset_drain_api::selected_hashset_drain_report(raw)
}

pub fn dead_hashset_drain_report(raw: &str) -> String {
    hashset_drain_api::dead_hashset_drain_report(raw)
}
