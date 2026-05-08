pub trait LabelRender {
    fn label(&self) -> &str;

    fn render_label(&self) -> String {
        format!("generic:{}", self.label())
    }
}

pub struct GenericRecord {
    label: String,
}

impl GenericRecord {
    pub fn new(raw: &str) -> Self {
        Self {
            label: raw.trim().to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-generic:{}", self.label)
    }
}

impl LabelRender for GenericRecord {
    fn label(&self) -> &str {
        &self.label
    }
}

pub fn render_with_bound<T: LabelRender>(value: &T) -> String {
    value.render_label()
}

pub fn selected_generic(raw: &str) -> String {
    let record = GenericRecord::new(raw);
    render_with_bound(&record)
}

pub fn dead_live_generic(raw: &str) -> String {
    GenericRecord::new(raw).dead_method()
}
