use opensourced::opensourced;

#[opensourced]
pub fn selected_vec_pop_report(raw: &str) -> String {
    vec_pop_api::selected_vec_pop_report(raw)
}

pub fn dead_vec_pop_report(raw: &str) -> String {
    vec_pop_api::dead_vec_pop_report(raw)
}
