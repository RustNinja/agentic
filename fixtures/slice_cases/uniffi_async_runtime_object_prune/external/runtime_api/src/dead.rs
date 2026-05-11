use runtime_support::dead_runtime_report;

pub async fn dead_async_runtime_status(raw: &str) -> String {
    dead_runtime_report(raw).await
}
