use opensourced::opensourced;

#[opensourced]
pub fn selected_lazy_report(raw: &str) -> String {
    lazy_api::selected_lazy_report(raw)
}

pub fn dead_lazy_report(raw: &str) -> String {
    lazy_api::dead_lazy_report(raw)
}
