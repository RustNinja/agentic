#![allow(dead_code)]

mod dead;
mod live;

pub use dead::{render_dead_surface, DeadSurface, DeadSurfaceTrait};
pub use live::{render_live_surface, LiveSurface, SurfaceRender};
