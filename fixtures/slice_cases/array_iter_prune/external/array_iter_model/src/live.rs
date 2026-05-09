#[derive(Clone, Debug)]
pub struct ArrayIterItem {
    value: String,
}

impl ArrayIterItem {
    pub fn live() -> Self {
        Self::new("live")
    }

    pub fn breakpoint() -> Self {
        Self::new("break")
    }

    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn is_live(&self) -> bool {
        self.value == "live"
    }

    pub fn is_break(&self) -> bool {
        self.value == "break"
    }

    pub fn bump(&mut self) -> &mut Self {
        self.value.push_str("-bumped");
        self
    }

    pub fn render_label(&self) -> String {
        format!("array-iter:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-array-iter:{}", self.value)
    }
}

fn array_iter_entries(raw: &str) -> [ArrayIterItem; 2] {
    [ArrayIterItem::live(), ArrayIterItem::new(raw)]
}

pub fn selected_array_iter(raw: &str) -> String {
    let entries = array_iter_entries(raw);
    entries
        .iter()
        .find(|item| item.is_live())
        .map(|item| item.render_label())
        .unwrap_or_else(|| "array-iter:missing".to_string())
}

pub fn dead_live_array_iter(raw: &str) -> String {
    ArrayIterItem::new(raw).dead_method()
}
