pub mod geometry;
pub mod point;
pub mod simplify;
pub mod smooth;
pub mod stroke;

pub use geometry::{
    arrow_mesh, circle_mesh, generate_mesh, generate_mesh_closed, get_bounds, hit_path, hit_shape,
    line_mesh, rect_mesh,
};
pub use point::Point;
pub use simplify::rdp;
pub use smooth::catmull_rom;
pub use stroke::{Stroke, compute_widths, pipeline};

#[cfg(feature = "wasm")]
pub mod wasm;

#[cfg(feature = "wasm")]
pub mod render;

#[cfg(feature = "wasm")]
pub mod state;

#[cfg(feature = "wasm")]
pub mod net;

#[cfg(feature = "wasm")]
pub mod util;

#[cfg(feature = "wasm")]
pub mod handlers;

#[cfg(feature = "wasm")]
pub mod js_helpers;
