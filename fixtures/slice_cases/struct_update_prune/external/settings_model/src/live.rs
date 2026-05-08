pub struct Settings {
    pub name: String,
    pub enabled: bool,
    pub retries: u8,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            name: "default".to_string(),
            enabled: true,
            retries: 3,
        }
    }
}

impl Settings {
    pub fn render(&self) -> String {
        format!("settings:{}:{}:{}", self.name, self.enabled, self.retries)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-settings:{}", self.name)
    }
}

pub fn selected_settings(raw: &str) -> String {
    let settings = Settings {
        name: raw.trim().to_string(),
        ..Settings::default()
    };
    settings.render()
}

pub fn dead_live_settings(raw: &str) -> String {
    Settings {
        name: raw.into(),
        ..Settings::default()
    }
    .dead_method()
}
