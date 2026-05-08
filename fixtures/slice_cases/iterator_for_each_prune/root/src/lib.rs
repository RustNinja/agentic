use opensourced::opensourced;

#[opensourced]
pub fn selected_for_each_report(raw: &str) -> String {
    foreach_api::selected_for_each_report(raw)
}

pub fn dead_for_each_report(raw: &str) -> String {
    foreach_api::dead_for_each_report(raw)
}
