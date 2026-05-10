pub mod theme;

pub fn dead_preview(health: &mobile_client::store::ServerHealthSnapshot) -> String {
    format!(
        "{}:{}",
        theme::accent().name(),
        mobile_client::store::dead_health_label(health)
    )
}

