use egui::{LayerId, Painter};

/// Uses egui painter to:
///
/// Render a grid and the creature on it, add bunch of visualizations if requested:
/// WIP
use super::super::dnaparser;

struct DnaGrid {
    painter: Painter,
}

impl DnaGrid {
    pub fn new(ctx: egui::Context, layer_id: LayerId, clip_rect: egui::Rect) -> Self {
        DnaGrid {
            painter: Painter::new(ctx, layer_id, clip_rect),
        }
    }
}
