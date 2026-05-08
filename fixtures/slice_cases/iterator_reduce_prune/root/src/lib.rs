use opensourced::opensourced;

#[opensourced]
pub fn selected_reduce_report(raw: &str) -> String {
    reduce_api::selected_reduce_report(raw)
}

pub fn dead_reduce_report(raw: &str) -> String {
    reduce_api::dead_reduce_report(raw)
}
