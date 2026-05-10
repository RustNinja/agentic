use opensourced::opensourced;

#[opensourced]
pub fn reconnect_summary(raw: &str) -> String {
    codex_client::build_reconnect(raw).summary()
}

pub fn dead_reconnect_summary(raw: &str) -> String {
    codex_client::dead_reconnect(raw)
}
