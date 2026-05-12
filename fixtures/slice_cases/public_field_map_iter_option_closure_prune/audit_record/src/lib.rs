use std::collections::HashMap;

#[derive(Clone, Eq, Hash, PartialEq)]
pub struct ThreadKey {
    pub server_id: String,
    pub thread_id: String,
    pub dead_key_note: Option<String>,
}

pub struct ThreadInfo {
    pub title: Option<String>,
    pub cwd: Option<String>,
    pub dead_info_note: Option<String>,
}

pub struct ThreadSnapshot {
    pub info: ThreadInfo,
    pub active_turn_id: Option<String>,
    pub dead_thread_note: Option<String>,
}

pub struct ServerSnapshot {
    pub display_name: &'static str,
    pub host: &'static str,
    pub port: u16,
    pub health: String,
    pub dead_server_note: Option<String>,
}

pub struct OtherDisplay {
    pub display_name: &'static str,
    pub dead_other_note: Option<String>,
}

pub struct AppSnapshot {
    pub servers: HashMap<String, ServerSnapshot>,
    pub threads: HashMap<ThreadKey, ThreadSnapshot>,
    pub other: OtherDisplay,
    pub unused_note: Option<String>,
}

pub fn snapshot(seed: u32) -> AppSnapshot {
    let server_id = format!("server-{seed}");
    let thread_key = ThreadKey {
        server_id: server_id.clone(),
        thread_id: format!("thread-{seed}"),
        dead_key_note: None,
    };
    let mut servers = HashMap::new();
    servers.insert(
        server_id.clone(),
        ServerSnapshot {
            display_name: "Server",
            host: "localhost",
            port: 9000,
            health: "ok".to_string(),
            dead_server_note: None,
        },
    );
    let mut threads = HashMap::new();
    threads.insert(
        thread_key,
        ThreadSnapshot {
            info: ThreadInfo {
                title: Some(format!("Thread {seed}")),
                cwd: Some("/tmp".to_string()),
                dead_info_note: None,
            },
            active_turn_id: Some("turn".to_string()),
            dead_thread_note: None,
        },
    );
    AppSnapshot {
        servers,
        threads,
        other: OtherDisplay {
            display_name: "Other",
            dead_other_note: None,
        },
        unused_note: None,
    }
}

pub fn dead_snapshot(seed: u32) -> AppSnapshot {
    let mut snapshot = snapshot(seed);
    snapshot.unused_note = Some("dead".to_string());
    snapshot
}
