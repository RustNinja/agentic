pub trait Reader {
    fn read(&self) -> String;

    fn dead_default(&self) -> String {
        "dead-default-reader".to_string()
    }
}

pub struct LiveReader {
    label: String,
}

impl LiveReader {
    pub fn new(raw: &str) -> Self {
        Self {
            label: raw.trim().to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-live-reader:{}", self.label)
    }
}

impl Reader for LiveReader {
    fn read(&self) -> String {
        format!("live-reader:{}", self.label)
    }
}

pub fn selected_reader(raw: &str) -> Box<dyn Reader> {
    Box::new(LiveReader::new(raw))
}

pub fn render_reader(reader: Box<dyn Reader>) -> String {
    reader.read()
}

pub fn dead_live_dyn_summary(raw: &str) -> String {
    LiveReader::new(raw).dead_method()
}

