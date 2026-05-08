pub fn selected_reduce_report(raw: &str) -> String {
    reduce_model::selected_reduce(raw)
}

pub fn dead_live_reduce_report(raw: &str) -> String {
    format!("dead-live-reduce:{raw}")
}
