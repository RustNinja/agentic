use trait_model::{render_dead_surface, DeadSurface};

pub fn dead_trait_report(raw: &str) -> String {
    let surface = DeadSurface::new(raw);
    format!("api-dead:{}", render_dead_surface(&surface))
}
