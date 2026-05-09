use opensourced::opensourced;

#[opensourced]
pub fn selected_matches_result_report(raw: &str) -> String {
    matches_result_api::selected_matches_result_report(raw)
}

pub fn dead_matches_result_report(raw: &str) -> String {
    matches_result_api::dead_matches_result_report(raw)
}
