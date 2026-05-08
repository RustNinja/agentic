use opensourced::opensourced;

#[opensourced]
pub fn selected_try_for_each_report(raw: &str) -> String {
    try_api::selected_try_for_each_report(raw)
}

pub fn dead_try_for_each_report(raw: &str) -> String {
    try_api::dead_try_for_each_report(raw)
}
