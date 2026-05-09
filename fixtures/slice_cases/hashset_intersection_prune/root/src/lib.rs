use opensourced::opensourced;

#[opensourced]
pub fn selected_hashset_intersection_report(raw: &str) -> String {
    hashset_intersection_api::selected_hashset_intersection_report(raw)
}

pub fn dead_hashset_intersection_report(raw: &str) -> String {
    hashset_intersection_api::dead_hashset_intersection_report(raw)
}
