pub fn selected_closure_return_report(raw: &str) -> String {
    closure_return_model::selected_closure_return(raw)
}

pub fn dead_live_closure_return_report(raw: &str) -> String {
    format!("dead-live-closure-return:{raw}")
}
