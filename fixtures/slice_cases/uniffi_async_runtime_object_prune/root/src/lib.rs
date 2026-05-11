use opensource_attr::opensourced;

#[opensourced]
pub async fn selected_async_runtime_status(raw: &str) -> String {
    runtime_api::selected_async_runtime_status(raw).await
}

pub async fn dead_async_runtime_status(raw: &str) -> String {
    runtime_api::dead_async_runtime_status(raw).await
}
