use stream_model::{selected_stream, StreamError};

pub fn selected_stream_report(raw: &str) -> String {
    selected_stream(raw)
        .map(|envelope| envelope.render())
        .unwrap_or_else(StreamError::render)
}
