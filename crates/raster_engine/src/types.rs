use image::Rgba;

/// Mode of output generation for 2.5D extrusion
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ExtrudeOutputMode {
    /// Single merged flat RGBA layer containing front face, extrusion body and drop shadow
    #[default]
    Merged,
    /// Three separate RGBA layers: (Front, Extrude Body, Drop Shadow)
    SplitLayers,
}

/// Parameters controlling 2.5D extrusion and stylization
#[derive(Debug, Clone, PartialEq)]
pub struct ExtrudeParams {
    /// Extrusion angle in degrees (0 = right, 90 = down, 180 = left, 270 = up)
    pub angle_deg: f32,
    /// Extrusion distance in pixels
    pub distance: f32,
    /// Optional override or tint color for the front face
    pub front_color: Option<Rgba<u8>>,
    /// Gradient color of the extrusion body near the front face
    pub body_gradient_start: Rgba<u8>,
    /// Gradient color of the extrusion body at the furthest depth
    pub body_gradient_end: Rgba<u8>,
    /// Inner bevel highlight radius in pixels (0.0 disables bevel)
    pub bevel_radius: f32,
    /// Highlight strength of the bevel (0.0 to 1.0)
    pub bevel_highlight: f32,
    /// Light source angle for shading in degrees (0..360)
    pub light_angle_deg: f32,
    /// Light intensity / shading contrast (0.0 to 1.0)
    pub light_intensity: f32,
    /// Drop shadow blur radius in pixels (0.0 disables shadow)
    pub shadow_blur: f32,
    /// Drop shadow opacity (0.0 to 1.0)
    pub shadow_opacity: f32,
    /// Additional shadow offset distance along angle
    pub shadow_distance: f32,
    /// Perspective convergence factor (0.0 = isometric/parallel, > 0.0 = tapers towards vanishing point in distance)
    pub perspective: f32,
    /// Output mode (merged or separate layers)
    pub output_mode: ExtrudeOutputMode,
}

impl Default for ExtrudeParams {
    fn default() -> Self {
        Self {
            angle_deg: 135.0, // bottom-right
            distance: 24.0,
            front_color: None,
            body_gradient_start: Rgba([230, 70, 70, 255]),
            body_gradient_end: Rgba([70, 20, 30, 255]),
            bevel_radius: 1.5,
            bevel_highlight: 0.6,
            light_angle_deg: 315.0, // top-left
            light_intensity: 0.35,
            shadow_blur: 8.0,
            shadow_opacity: 0.5,
            shadow_distance: 12.0,
            perspective: 0.0,
            output_mode: ExtrudeOutputMode::Merged,
        }
    }
}

impl ExtrudeParams {
    /// Proportionally scale all pixel-distance parameters (distance, bevel, shadow)
    /// to maintain 100% visual parity between preview thumbnail and full-canvas render.
    pub fn scaled(&self, factor: f32) -> Self {
        let mut scaled = self.clone();
        scaled.distance = (self.distance * factor).max(0.0);
        scaled.bevel_radius = (self.bevel_radius * factor).max(0.0);
        scaled.shadow_blur = (self.shadow_blur * factor).max(0.0);
        scaled.shadow_distance = (self.shadow_distance * factor).max(0.0);
        scaled.perspective = self.perspective;
        scaled
    }
}
