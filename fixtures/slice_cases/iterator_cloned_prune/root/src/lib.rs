use opensourced::opensourced;

#[opensourced]
pub fn selected_cloned_report(raw: &str) -> String {
    cloned_api::selected_cloned_report(raw)
}

pub fn dead_cloned_report(raw: &str) -> String {
    cloned_api::dead_cloned_report(raw)
}
