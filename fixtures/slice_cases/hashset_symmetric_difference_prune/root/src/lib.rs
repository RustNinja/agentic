use opensourced::opensourced;

#[opensourced]
pub fn selected_hashset_symmetric_difference_report(raw: &str) -> String {
    hashset_symmetric_difference_api::selected_hashset_symmetric_difference_report(raw)
}

pub fn dead_hashset_symmetric_difference_report(raw: &str) -> String {
    hashset_symmetric_difference_api::dead_hashset_symmetric_difference_report(raw)
}
