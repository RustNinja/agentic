pub struct DeadCommand {
    raw: String,
}

impl DeadCommand {
    pub fn new(raw: &str) -> Self {
        Self {
            raw: raw.to_string(),
        }
    }
}

pub async fn dead_command(raw: &str) -> String {
    format!("dead-command:{}", DeadCommand::new(raw).raw)
}
