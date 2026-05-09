use opensourced::opensourced;

#[opensourced]
pub fn selected_vec_drain_report(raw: &str) -> String {
    vec_drain_api::selected_vec_drain_report(raw)
}

pub fn dead_vec_drain_report(raw: &str) -> String {
    vec_drain_api::dead_vec_drain_report(raw)
}
