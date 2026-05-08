pub fn selected_dispatch_report(method: &str, payload: &str) -> String {
    dispatch_model::selected_dispatch(method, payload)
}

pub fn dead_live_dispatch_report(raw: &str) -> String {
    format!("dead-live-dispatch:{raw}")
}
