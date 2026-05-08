use opensourced::opensourced;

#[opensourced]
pub fn selected_dispatch_report(method: &str, payload: &str) -> String {
    dispatch_api::selected_dispatch_report(method, payload)
}

pub fn dead_dispatch_report(raw: &str) -> String {
    dispatch_api::dead_dispatch_report(raw)
}
