mod private;
pub mod snapshot;

pub use private::dead_private_label;
pub use snapshot::{AppSnapshot, DeadHealthSnapshot, ServerHealthSnapshot};

pub fn dead_health_label(health: &ServerHealthSnapshot) -> &'static str {
    match health {
        ServerHealthSnapshot::Connected => "dead-connected",
        _ => dead_private_label(),
    }
}

