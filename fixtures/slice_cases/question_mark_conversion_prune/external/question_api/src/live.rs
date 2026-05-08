pub fn selected_question_report(raw: &str) -> String {
    match question_model::selected_question(raw) {
        Ok(value) => value,
        Err(err) => err.render(),
    }
}

pub fn dead_live_question_report(raw: &str) -> String {
    format!("dead-live-question:{raw}")
}
