pub struct SliceWindowsFilterMapPayload {
    value: String,
}

impl SliceWindowsFilterMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("slice-windows-filter-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-slice-windows-filter-map:{}", self.value)
    }
}

pub fn selected_slice_windows_filter_map(raw: &str) -> String {
    let items = vec![
        SliceWindowsFilterMapPayload::new(raw),
        SliceWindowsFilterMapPayload::new("tail"),
    ];
    items
        .windows(2)
        .filter_map(|window| window.first().map(|payload| payload.render_label()))
        .collect::<Vec<_>>()
        .join("|")
}

pub fn dead_live_slice_windows_filter_map(raw: &str) -> String {
    SliceWindowsFilterMapPayload::new(raw).unused_label()
}
