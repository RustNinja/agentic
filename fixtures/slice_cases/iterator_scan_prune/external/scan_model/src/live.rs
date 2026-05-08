pub struct ScanItem {
    value: String,
}

impl ScanItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render(&self) -> String {
        format!("scan:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-scan:{}", self.value)
    }
}

pub struct ScanState {
    seen: usize,
}

impl ScanState {
    pub fn new() -> Self {
        Self { seen: 0 }
    }

    pub fn accept(&mut self, item: &ScanItem) -> Option<String> {
        self.seen += 1;
        (self.seen <= 3).then(|| item.render())
    }

    pub fn dead_method(&self) -> String {
        format!("dead-scan-state:{}", self.seen)
    }
}

fn build_items(raw: &str) -> Vec<ScanItem> {
    raw.split(',').map(ScanItem::new).collect()
}

pub fn selected_scan(raw: &str) -> String {
    build_items(raw)
        .iter()
        .scan(ScanState::new(), |state, item| state.accept(item))
        .collect::<Vec<_>>()
        .join("|")
}

pub fn dead_live_scan(raw: &str) -> String {
    ScanItem::new(raw).dead_method()
}
