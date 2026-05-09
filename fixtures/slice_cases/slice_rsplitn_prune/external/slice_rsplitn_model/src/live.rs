#[derive(Clone, Debug)]
pub struct SliceRsplitnItem {
    value: String,
}

impl SliceRsplitnItem {
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
        format!("slice-rsplitn:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-slice-rsplitn:{}", self.value)
    }
}

fn slice_rsplitn_entries(raw: &str) -> Vec<SliceRsplitnItem> {
    vec![SliceRsplitnItem::live(), SliceRsplitnItem::breakpoint(), SliceRsplitnItem::new(raw)]
}

pub fn selected_slice_rsplitn(raw: &str) -> String {
    let entries = slice_rsplitn_entries(raw);
    entries
        .rsplitn(2, |item| item.is_break())
        .map(|group| group.iter().map(|item| item.render_label()).collect::<Vec<_>>().join("+"))
        .collect::<Vec<_>>()
        .join("|")
}

pub fn dead_live_slice_rsplitn(raw: &str) -> String {
    SliceRsplitnItem::new(raw).dead_method()
}
