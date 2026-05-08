use opensourced::opensourced;

#[opensourced]
pub fn selected_stream_report(raw: &str) -> String {
    stream_api::selected_stream_report(raw)
}

pub fn dead_stream_report(raw: &str) -> String {
    stream_api::dead_stream_report(raw)
}
