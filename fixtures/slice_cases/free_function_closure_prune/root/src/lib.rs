use opensourced::opensourced;

#[opensourced]
pub fn selected_free_closure_report(raw: &str) -> String {
    closure_api::selected_free_closure_report(raw)
}

pub fn dead_free_closure_report(raw: &str) -> String {
    closure_api::dead_free_closure_report(raw)
}
