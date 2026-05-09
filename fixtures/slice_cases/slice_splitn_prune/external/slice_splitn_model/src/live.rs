#[derive(Clone, Debug)]
pub struct SliceSplitnItem {
    value: String,
}

impl SliceSplitnItem {
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
        format!("slice-splitn:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-slice-splitn:{}", self.value)
    }
}

fn slice_splitn_entries(raw: &str) -> Vec<SliceSplitnItem> {
    vec![SliceSplitnItem::live(), SliceSplitnItem::breakpoint(), SliceSplitnItem::new(raw)]
}

pub fn selected_slice_splitn(raw: &str) -> String {
    let entries = slice_splitn_entries(raw);
    entries
        .splitn(2, |item| item.is_break())
        .map(|group| group.iter().map(|item| item.render_label()).collect::<Vec<_>>().join("+"))
        .collect::<Vec<_>>()
        .join("|")
}

pub fn dead_live_slice_splitn(raw: &str) -> String {
    SliceSplitnItem::new(raw).dead_method()
}
