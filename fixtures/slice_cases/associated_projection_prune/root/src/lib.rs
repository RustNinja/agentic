use opensourced::opensourced;

#[opensourced]
pub fn selected_projection_report(raw: &str) -> String {
    projection_api::selected_projection_report(raw)
}

pub fn dead_projection_report(raw: &str) -> String {
    projection_api::dead_projection_report(raw)
}
