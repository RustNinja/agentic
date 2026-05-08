use opensourced::opensourced;

#[opensourced]
pub fn selected_enumerate_report(raw: &str) -> String {
    enumerate_api::selected_enumerate_report(raw)
}

pub fn dead_enumerate_report(raw: &str) -> String {
    enumerate_api::dead_enumerate_report(raw)
}
