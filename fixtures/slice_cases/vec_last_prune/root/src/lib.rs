use opensourced::opensourced;

#[opensourced]
pub fn selected_vec_last_report(raw: &str) -> String {
    vec_last_api::selected_vec_last_report(raw)
}

pub fn dead_vec_last_report(raw: &str) -> String {
    vec_last_api::dead_vec_last_report(raw)
}
