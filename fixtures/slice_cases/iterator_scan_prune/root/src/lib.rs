use opensourced::opensourced;

#[opensourced]
pub fn selected_scan_report(raw: &str) -> String {
    scan_api::selected_scan_report(raw)
}

pub fn dead_scan_report(raw: &str) -> String {
    scan_api::dead_scan_report(raw)
}
