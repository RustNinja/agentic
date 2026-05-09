use opensourced::opensourced;

#[opensourced]
pub fn selected_vec_retain_mut_report(raw: &str) -> String {
    vec_retain_mut_api::selected_vec_retain_mut_report(raw)
}

pub fn dead_vec_retain_mut_report(raw: &str) -> String {
    vec_retain_mut_api::dead_vec_retain_mut_report(raw)
}
