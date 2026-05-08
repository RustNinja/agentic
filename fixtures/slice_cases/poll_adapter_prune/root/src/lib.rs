use opensourced::opensourced;

#[opensourced]
pub fn selected_poll_report(raw: &str) -> String {
    poll_api::selected_poll_report(raw)
}

pub fn dead_poll_report(raw: &str) -> String {
    poll_api::dead_poll_report(raw)
}
