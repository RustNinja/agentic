const DEAD_ASSET: &str = include_str!("assets/dead.txt");

pub struct DeadAssetReport {
    label: String,
}

impl DeadAssetReport {
    pub fn new(raw: &str) -> Self {
        Self {
            label: raw.to_string(),
        }
    }

    pub fn render_dead(self) -> String {
        format!("dead-asset:{DEAD_ASSET}:{}", self.label)
    }
}

pub fn build_dead_asset(raw: &str) -> DeadAssetReport {
    DeadAssetReport::new(raw)
}
