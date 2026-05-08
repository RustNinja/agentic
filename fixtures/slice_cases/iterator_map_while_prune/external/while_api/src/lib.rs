mod dead;
mod live;

pub use live::selected_map_while_report;

pub fn dead_map_while_report(raw: &str) -> String {
    dead::dead_map_while_report(raw)
}
