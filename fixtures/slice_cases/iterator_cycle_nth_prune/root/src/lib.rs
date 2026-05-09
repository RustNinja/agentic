use opensourced::opensourced;

#[opensourced]
pub fn selected_cycle_nth_report(raw: &str) -> String {
    cycle_nth_api::selected_cycle_nth_report(raw)
}

pub fn dead_cycle_nth_report(raw: &str) -> String {
    cycle_nth_api::dead_cycle_nth_report(raw)
}
