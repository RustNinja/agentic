use stream_model::{dead_stream, DeadEnvelope};

pub fn dead_stream_report(raw: &str) -> String {
    let envelope: DeadEnvelope = dead_stream(raw);
    envelope.render_dead()
}
