use opensourced::opensourced;

#[opensourced]
pub fn selected_closure_report(raw: &str) -> String {
    closure_api::selected_closure_report(raw)
}

pub fn dead_closure_report(raw: &str) -> String {
    closure_api::dead_closure_report(raw)
}
