pub struct MatchesGuardPayload {
    value: String,
}

impl MatchesGuardPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn accepts(&self) -> bool {
        self.render_label().contains("ready")
    }

    pub fn render_label(&self) -> String {
        format!("matches-guard:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-matches-guard:{}", self.value)
    }
}

fn matches_guard_payload(raw: &str) -> Option<MatchesGuardPayload> {
    if raw.trim().is_empty() {
        None
    } else {
        Some(MatchesGuardPayload::new(raw))
    }
}

pub fn selected_matches_guard(raw: &str) -> String {
    let payload = matches_guard_payload(raw);
    if matches!(payload.as_ref(), Some(candidate) if candidate.accepts()) {
        payload
            .map(|candidate| candidate.render_label())
            .unwrap_or_else(|| "matches-guard:missing".to_string())
    } else {
        "matches-guard:rejected".to_string()
    }
}

pub fn dead_live_matches_guard(raw: &str) -> String {
    MatchesGuardPayload::new(raw).dead_method()
}
