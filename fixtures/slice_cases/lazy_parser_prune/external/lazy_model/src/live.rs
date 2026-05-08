use std::sync::LazyLock;

static LIVE_PARSER: LazyLock<LazyParser> = LazyLock::new(|| LazyParser::new("lazy"));

pub struct LazyParser {
    prefix: &'static str,
}

impl LazyParser {
    pub fn new(prefix: &'static str) -> Self {
        Self { prefix }
    }

    pub fn parse(&self, raw: &str) -> LazyToken {
        LazyToken {
            label: format!("{}:{}", self.prefix, raw.trim()),
        }
    }

    pub fn dead_method(&self, raw: &str) -> String {
        format!("dead-lazy:{raw}")
    }
}

pub struct LazyToken {
    label: String,
}

impl LazyToken {
    pub fn render(&self) -> String {
        self.label.clone()
    }

    pub fn dead_method(&self) -> String {
        format!("dead-token:{}", self.label)
    }
}

pub fn selected_lazy(raw: &str) -> String {
    LIVE_PARSER.parse(raw).render()
}

pub fn dead_live_lazy(raw: &str) -> String {
    LazyParser::new("dead").dead_method(raw)
}
