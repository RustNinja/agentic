use opensourced::opensourced;

#[opensourced]
pub fn selected_for_loop_report(raw: &str) -> String {
    for_loop_api::selected_for_loop_report(raw)
}

pub fn dead_for_loop_report(raw: &str) -> String {
    for_loop_api::dead_for_loop_report(raw)
}
