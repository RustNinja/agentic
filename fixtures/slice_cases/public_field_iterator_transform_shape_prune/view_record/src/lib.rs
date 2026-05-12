pub struct ViewRecord {
    pub title: String,
    pub score: u32,
    pub loop_title: String,
    pub dead_view_note: Option<String>,
}

pub struct FilterRecord {
    pub code: String,
    pub weight: u32,
    pub dead_filter_note: Option<String>,
}

pub struct ChildView {
    pub child_label: String,
    pub child_weight: u32,
    pub dead_child_view_note: Option<String>,
}
