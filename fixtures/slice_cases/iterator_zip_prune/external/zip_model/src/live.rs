pub struct ZipLeft {
    value: String,
}

impl ZipLeft {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_with(&self, right: &ZipRight) -> String {
        format!("zip:{}:{}", self.value, right.render_label())
    }

    pub fn dead_method(&self) -> String {
        format!("dead-zip-left:{}", self.value)
    }
}

pub struct ZipRight {
    value: String,
}

impl ZipRight {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("right:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-zip-right:{}", self.value)
    }
}

fn left_items(raw: &str) -> Vec<ZipLeft> {
    raw.split(',').map(ZipLeft::new).collect()
}

fn right_items(raw: &str) -> Vec<ZipRight> {
    raw.split(',').rev().map(ZipRight::new).collect()
}

pub fn selected_zip(raw: &str) -> String {
    let left = left_items(raw);
    let right = right_items(raw);
    left.iter()
        .zip(right.iter())
        .map(|(left, right)| left.render_with(right))
        .collect::<Vec<_>>()
        .join("|")
}

pub fn dead_live_zip(raw: &str) -> String {
    ZipLeft::new(raw).dead_method()
}
