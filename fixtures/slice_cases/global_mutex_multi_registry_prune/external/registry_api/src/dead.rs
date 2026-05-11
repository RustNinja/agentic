pub fn dead_registry_lookup(raw: &str) -> String {
    registry_store::list_entries()
        .into_iter()
        .find(|entry| entry.render().contains(raw))
        .map(|entry| entry.dead_render())
        .unwrap_or_else(|| "dead-list-empty".to_string())
}
