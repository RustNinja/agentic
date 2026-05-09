use opensourced::opensourced;

#[opensourced]
pub fn selected_vec_first_report(raw: &str) -> String {
    vec_first_api::selected_vec_first_report(raw)
}

pub fn dead_vec_first_report(raw: &str) -> String {
    vec_first_api::dead_vec_first_report(raw)
}
