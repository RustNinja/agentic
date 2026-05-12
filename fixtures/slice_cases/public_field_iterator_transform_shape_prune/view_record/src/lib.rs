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

pub struct LocalView {
    pub local_title: String,
    pub local_score: u32,
    pub dead_local_note: Option<String>,
}

pub struct LazyView {
    pub lazy_title: String,
    pub lazy_score: u32,
    pub dead_lazy_note: Option<String>,
}

pub struct ReturnedView {
    pub returned_title: String,
    pub returned_score: u32,
    pub dead_returned_note: Option<String>,
}

pub struct MethodView {
    pub method_title: String,
    pub method_score: u32,
    pub dead_method_note: Option<String>,
}

pub struct ParamView {
    pub param_title: String,
    pub param_score: u32,
    pub dead_param_note: Option<String>,
}
