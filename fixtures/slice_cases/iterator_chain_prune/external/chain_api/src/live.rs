pub fn selected_chain_report(raw: &str) -> String {
    chain_model::selected_chain(raw)
}

pub fn dead_live_chain_report(raw: &str) -> String {
    format!("dead-live-chain:{raw}")
}
