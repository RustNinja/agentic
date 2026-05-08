use opensourced::opensourced;

#[opensourced]
pub fn selected_question_report(raw: &str) -> String {
    question_api::selected_question_report(raw)
}

pub fn dead_question_report(raw: &str) -> String {
    question_api::dead_question_report(raw)
}
