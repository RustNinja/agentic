use opensourced::opensourced;

#[opensourced]
pub fn selected_transpose_report(raw: &str) -> String {
    transpose_api::selected_transpose_report(raw)
}

pub fn dead_transpose_report(raw: &str) -> String {
    transpose_api::dead_transpose_report(raw)
}
