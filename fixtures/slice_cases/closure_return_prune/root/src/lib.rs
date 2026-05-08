use opensourced::opensourced;

#[opensourced]
pub fn selected_closure_return_report(raw: &str) -> String {
    closure_return_api::selected_closure_return_report(raw)
}

pub fn dead_closure_return_report(raw: &str) -> String {
    closure_return_api::dead_closure_return_report(raw)
}
