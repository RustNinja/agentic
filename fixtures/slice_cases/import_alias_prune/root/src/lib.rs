use opensourced::opensourced;

#[opensourced]
pub fn selected_report(raw: &str) -> String {
    gateway::selected_report(raw)
}

pub fn dead_report(raw: &str) -> String {
    gateway::dead_report(raw)
}
