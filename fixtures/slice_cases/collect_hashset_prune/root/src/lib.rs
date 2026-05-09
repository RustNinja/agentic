use opensourced::opensourced;

#[opensourced]
pub fn selected_collect_hashset_report(raw: &str) -> String {
    collect_hashset_api::selected_collect_hashset_report(raw)
}

pub fn dead_collect_hashset_report(raw: &str) -> String {
    collect_hashset_api::dead_collect_hashset_report(raw)
}
