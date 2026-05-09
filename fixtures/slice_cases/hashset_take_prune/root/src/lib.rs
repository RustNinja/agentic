use opensourced::opensourced;

#[opensourced]
pub fn selected_hashset_take_report(raw: &str) -> String {
    hashset_take_api::selected_hashset_take_report(raw)
}

pub fn dead_hashset_take_report(raw: &str) -> String {
    hashset_take_api::dead_hashset_take_report(raw)
}
