use opensourced::opensourced;

#[opensourced]
pub fn selected_hashset_union_report(raw: &str) -> String {
    hashset_union_api::selected_hashset_union_report(raw)
}

pub fn dead_hashset_union_report(raw: &str) -> String {
    hashset_union_api::dead_hashset_union_report(raw)
}
