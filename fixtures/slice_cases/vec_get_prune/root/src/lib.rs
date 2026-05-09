use opensourced::opensourced;

#[opensourced]
pub fn selected_vec_get_report(raw: &str) -> String {
    vec_get_api::selected_vec_get_report(raw)
}

pub fn dead_vec_get_report(raw: &str) -> String {
    vec_get_api::dead_vec_get_report(raw)
}
