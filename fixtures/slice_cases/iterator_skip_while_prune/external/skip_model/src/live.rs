pub struct SkipItem {
    value: String,
}

impl SkipItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn skip_prefix(&self) -> bool {
        self.value == "skip"
    }

    pub fn render(&self) -> String {
        format!("skip:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-skip:{}", self.value)
    }
}

fn build_items(raw: &str) -> Vec<SkipItem> {
    raw.split(',').map(SkipItem::new).collect()
}

pub fn selected_skip_while(raw: &str) -> String {
    build_items(raw)
        .iter()
        .skip_while(|item| item.skip_prefix())
        .map(|item| item.render())
        .collect::<Vec<_>>()
        .join("|")
}

pub fn dead_live_skip_while(raw: &str) -> String {
    SkipItem::new(raw).dead_method()
}
