use audit_record::AppSnapshot;

pub struct RecentSession {
    pub title: String,
    pub server_name: String,
    pub cwd: String,
    pub is_active: bool,
}

pub fn selected_summary(seed: u32) -> String {
    let snapshot = audit_record::snapshot(seed);
    let mut sessions: Vec<_> = snapshot
        .threads
        .iter()
        .map(|(key, thread)| {
            let server_name = snapshot
                .servers
                .get(&key.server_id)
                .map(|server| server.display_name.to_string())
                .unwrap_or_else(|| key.server_id.clone());
            RecentSession {
                title: thread
                    .info
                    .title
                    .clone()
                    .unwrap_or_else(|| "Untitled".to_string()),
                server_name,
                cwd: thread.info.cwd.clone().unwrap_or_default(),
                is_active: thread.active_turn_id.is_some(),
            }
        })
        .collect();
    sessions.sort_by(|left, right| {
        right
            .is_active
            .cmp(&left.is_active)
            .then_with(|| left.title.cmp(&right.title))
    });
    let server_endpoints = snapshot
        .servers
        .values()
        .map(|server| format!("{}:{}:{}", server.host, server.port, server.display_name))
        .collect::<Vec<_>>()
        .join(",");
    format!(
        "{}:{}:{}:{}:{}",
        sessions.len(),
        snapshot.other.display_name,
        sessions[0].server_name,
        sessions[0].cwd,
        server_endpoints
    )
}

pub fn dead_summary(seed: u32) -> String {
    let snapshot = audit_record::dead_snapshot(seed);
    snapshot
        .threads
        .values()
        .filter_map(|thread| thread.dead_thread_note.clone())
        .chain(
            snapshot
                .servers
                .values()
                .filter_map(|server| server.dead_server_note.clone()),
        )
        .chain(snapshot.unused_note)
        .collect::<Vec<_>>()
        .join(",")
}

pub fn from_snapshot(snapshot: &AppSnapshot) -> usize {
    snapshot.threads.len()
}
