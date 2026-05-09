#[derive(Clone, Debug)]
pub struct SliceWindowsItem {
    value: String,
}

impl SliceWindowsItem {
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
        format!("slice-windows:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-slice-windows:{}", self.value)
    }
}

fn slice_windows_entries(raw: &str) -> Vec<SliceWindowsItem> {
    vec![SliceWindowsItem::live(), SliceWindowsItem::breakpoint(), SliceWindowsItem::new(raw)]
}

pub fn selected_slice_windows(raw: &str) -> String {
    let entries = slice_windows_entries(raw);
    entries
        .windows(2)
        .map(|window| window.iter().map(|item| item.render_label()).collect::<Vec<_>>().join("+"))
        .collect::<Vec<_>>()
        .join("|")
}

pub fn dead_live_slice_windows(raw: &str) -> String {
    SliceWindowsItem::new(raw).dead_method()
}
