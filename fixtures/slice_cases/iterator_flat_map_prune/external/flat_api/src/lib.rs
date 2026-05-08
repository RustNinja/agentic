mod dead;
mod live;

pub use live::selected_flat_map_report;

pub fn dead_flat_map_report(raw: &str) -> String {
    dead::dead_flat_map_report(raw)
}
