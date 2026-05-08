pub struct FoldPart {
    value: String,
}

impl FoldPart {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render(&self) -> String {
        format!("part:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-part:{}", self.value)
    }
}

pub struct FoldSummary {
    parts: Vec<String>,
}

impl FoldSummary {
    pub fn new() -> Self {
        Self { parts: Vec::new() }
    }

    pub fn push(mut self, value: String) -> Self {
        self.parts.push(value);
        self
    }

    pub fn render(self) -> String {
        self.parts.join("|")
    }

    pub fn dead_method(self) -> String {
        format!("dead-summary:{}", self.parts.len())
    }
}

fn build_parts(raw: &str) -> Vec<FoldPart> {
    raw.split(',').map(FoldPart::new).collect()
}

pub fn selected_fold(raw: &str) -> String {
    build_parts(raw)
        .iter()
        .fold(FoldSummary::new(), |summary, part| summary.push(part.render()))
        .render()
}

pub fn dead_live_fold(raw: &str) -> String {
    FoldPart::new(raw).dead_method()
}
