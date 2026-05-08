pub async fn dead_command_report(raw: &str) -> String {
    command_runtime::dead_command(raw).await
}
