#[derive(Clone, Debug)]
pub struct SliceIterMutItem {
    value: String,
}

impl SliceIterMutItem {
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
        format!("slice-iter-mut:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-slice-iter-mut:{}", self.value)
    }
}

fn slice_iter_mut_entries(raw: &str) -> Vec<SliceIterMutItem> {
    vec![SliceIterMutItem::live(), SliceIterMutItem::breakpoint(), SliceIterMutItem::new(raw)]
}

pub fn selected_slice_iter_mut(raw: &str) -> String {
    let mut entries = slice_iter_mut_entries(raw);
    entries
        .iter_mut()
        .map(|item| item.bump().render_label())
        .collect::<Vec<_>>()
        .join("|")
}

pub fn dead_live_slice_iter_mut(raw: &str) -> String {
    SliceIterMutItem::new(raw).dead_method()
}
