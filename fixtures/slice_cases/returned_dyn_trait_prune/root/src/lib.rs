use opensourced::opensourced;

#[opensourced]
pub fn selected_dyn_report(raw: &str) -> String {
    dyn_api::selected_dyn_report(raw)
}

pub fn dead_dyn_report(raw: &str) -> String {
    dyn_api::dead_dyn_report(raw)
}

