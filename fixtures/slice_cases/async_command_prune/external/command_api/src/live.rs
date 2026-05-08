pub async fn selected_command_report(raw: &str) -> String {
    command_runtime::selected_command(raw).await
}

pub async fn dead_live_command_report(raw: &str) -> String {
    format!("dead-live-command:{raw}")
}
