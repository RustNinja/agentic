use opensourced::opensourced;

#[opensourced]
pub fn selected_nth_report(raw: &str) -> String {
    nth_api::selected_nth_report(raw)
}

pub fn dead_nth_report(raw: &str) -> String {
    nth_api::dead_nth_report(raw)
}
