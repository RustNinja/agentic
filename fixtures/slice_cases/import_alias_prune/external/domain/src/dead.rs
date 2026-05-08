use helper::{dead_helper, DeadHelper};

pub struct DeadModel {
    helper: DeadHelper,
}

impl DeadModel {
    pub fn new(raw: &str) -> Self {
        Self {
            helper: dead_helper(raw),
        }
    }

    pub fn render_dead(self) -> String {
        format!("dead-domain:{}", self.helper.render())
    }
}

pub fn build_dead_model(raw: &str) -> DeadModel {
    DeadModel::new(raw)
}

pub fn dead_prefix() -> &'static str {
    "dead"
}
