use opensourced::opensourced;

#[opensourced]
pub fn reconnect_callback_summary(raw: &str) -> String {
    codex_client::selected_reconnect(raw)
}

pub fn dead_reconnect_callback_summary(raw: &str) -> String {
    codex_client::dead_reconnect(raw)
}
