use opensourced::opensourced;

#[opensourced]
pub fn selected_peekable_nth_report(raw: &str) -> String {
    peekable_nth_api::selected_peekable_nth_report(raw)
}

pub fn dead_peekable_nth_report(raw: &str) -> String {
    peekable_nth_api::dead_peekable_nth_report(raw)
}
