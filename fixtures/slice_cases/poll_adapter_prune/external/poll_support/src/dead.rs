pub struct DeadPoller {
    raw: String,
}

pub fn dead_poll(raw: &str) -> String {
    format!("dead-poll:{}", DeadPoller { raw: raw.into() }.raw)
}
