pub fn selected_registry_lookup(raw: &str) -> String {
    registry_store::register_entry("live", raw);
    registry_store::lookup_entry("live")
        .map(|entry| entry.render())
        .unwrap_or_else(|| "missing".to_string())
}

pub fn dead_live_registry_lookup(raw: &str) -> String {
    registry_store::remove_entry(raw)
        .map(|entry| entry.dead_render())
        .unwrap_or_else(|| "dead-missing".to_string())
}
