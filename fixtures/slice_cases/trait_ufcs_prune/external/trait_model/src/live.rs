pub trait SurfaceRender {
    const PREFIX: &'static str;

    fn score(&self) -> u32;

    fn render(&self) -> String {
        format!("{}:{}", Self::PREFIX, self.score())
    }
}

pub struct LiveSurface {
    value: u32,
}

impl LiveSurface {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().len() as u32,
        }
    }

    pub fn dead_method(&self) -> u32 {
        self.value + 99
    }
}

impl SurfaceRender for LiveSurface {
    const PREFIX: &'static str = "live";

    fn score(&self) -> u32 {
        self.value + 1
    }
}

pub fn render_live_surface(surface: &LiveSurface) -> String {
    <LiveSurface as SurfaceRender>::render(surface)
}

pub fn dead_live_surface(raw: &str) -> u32 {
    LiveSurface::new(raw).dead_method()
}
