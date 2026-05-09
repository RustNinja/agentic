use opensourced::opensourced;

#[opensourced]
pub fn selected_result_match_report(raw: &str) -> String {
    result_match_api::selected_result_match_report(raw)
}

pub fn dead_result_match_report(raw: &str) -> String {
    result_match_api::dead_result_match_report(raw)
}
