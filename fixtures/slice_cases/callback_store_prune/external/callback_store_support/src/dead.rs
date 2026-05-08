pub struct DeadCallbackStore {
    raw: String,
}

pub fn dead_callback_store(raw: &str) -> String {
    format!(
        "dead-callback-store:{}",
        DeadCallbackStore { raw: raw.into() }.raw
    )
}
