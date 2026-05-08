use trait_model::{render_live_surface, LiveSurface};

pub fn selected_trait_report(raw: &str) -> String {
    let surface = LiveSurface::new(raw);
    render_live_surface(&surface)
}

pub fn dead_live_trait_report(raw: &str) -> String {
    format!("dead-live:{}", raw)
}
