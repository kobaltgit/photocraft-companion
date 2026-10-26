pub mod bevel;
pub mod extrude;
pub mod shading;
pub mod shadow;
pub mod types;

pub use extrude::{
    calculate_content_bounds, calculate_preview_scale, render_extrude_2_5d, render_extrude_preview,
    render_extrude_split,
};
pub use types::{ExtrudeOutputMode, ExtrudeParams};
