use opensourced::opensourced;

#[opensourced]
pub fn selected_rev_last_report(raw: &str) -> String {
    rev_last_api::selected_rev_last_report(raw)
}

pub fn dead_rev_last_report(raw: &str) -> String {
    rev_last_api::dead_rev_last_report(raw)
}
