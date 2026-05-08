pub trait DeadSurfaceTrait {
    fn dead_score(&self) -> u32;

    fn dead_render(&self) -> String {
        format!("dead:{}", self.dead_score())
    }
}

pub struct DeadSurface {
    value: u32,
}

impl DeadSurface {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.len() as u32,
        }
    }
}

impl DeadSurfaceTrait for DeadSurface {
    fn dead_score(&self) -> u32 {
        self.value + 99
    }
}

pub fn render_dead_surface(surface: &DeadSurface) -> String {
    <DeadSurface as DeadSurfaceTrait>::dead_render(surface)
}
