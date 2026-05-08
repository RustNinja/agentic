pub fn selected_callback_store_report(raw: &str) -> String {
    callback_store_support::selected_callback_store(raw)
}

pub fn dead_live_callback_store_report(raw: &str) -> String {
    format!("dead-live-callback-store:{raw}")
}
