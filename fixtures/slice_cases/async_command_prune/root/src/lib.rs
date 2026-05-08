use opensourced::opensourced;

#[opensourced]
pub async fn selected_command_report(raw: &str) -> String {
    command_api::selected_command_report(raw).await
}

pub async fn dead_command_report(raw: &str) -> String {
    command_api::dead_command_report(raw).await
}
