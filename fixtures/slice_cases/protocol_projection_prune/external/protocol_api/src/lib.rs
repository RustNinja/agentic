mod dead;
mod live;

pub use live::selected_protocol_report;

pub fn dead_protocol_report(raw: &str) -> String {
    dead::dead_protocol_report(raw)
}
