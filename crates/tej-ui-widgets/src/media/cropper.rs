//! Vector Signature Pad and File Dropzone.
//! Styled with Tej's dashed drag borders, upload cloud icons, and clear button triggers.

use tej_ui_hal::Color;
use tej_ui_layout::Rect;

/// A 2D point captured in a vector signature stroke.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StrokePoint {
    pub x: f32,
    pub y: f32,
    pub pressure: f32,
}

/// A continuous vector stroke.
#[derive(Debug, Clone, PartialEq)]
pub struct SignatureStroke {
    pub points: Vec<StrokePoint>,
    pub stroke_width: f32,
    pub stroke_color: Color,
}

/// State for the signature pad component.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct SignaturePadState {
    pub strokes: Vec<SignatureStroke>,
    pub is_drawing: bool,
}

impl SignaturePadState {
    pub fn begin_stroke(&mut self, pt: StrokePoint, color: Color, width: f32) {
        self.is_drawing = true;
        self.strokes.push(SignatureStroke {
            points: vec![pt],
            stroke_width: width,
            stroke_color: color,
        });
    }

    pub fn append_point(&mut self, pt: StrokePoint) {
        if self.is_drawing {
            if let Some(stroke) = self.strokes.last_mut() {
                stroke.points.push(pt);
            }
        }
    }

    pub fn end_stroke(&mut self) {
        self.is_drawing = false;
    }

    pub fn clear(&mut self) {
        self.strokes.clear();
        self.is_drawing = false;
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.strokes.is_empty()
    }
}

/// State for a file upload dropzone.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct DropzoneState {
    pub is_drag_over: bool,
    pub accepted_files: Vec<String>,
}

/// Tej styled Dropzone component.
#[derive(Debug, Clone, PartialEq)]
pub struct Dropzone {
    pub bounds: Rect,
    pub background: Color,
    pub drag_over_background: Color,
    pub border_color: Color,
    pub corner_radius: f32,
}

impl Default for Dropzone {
    fn default() -> Self {
        Self {
            bounds: Rect::ZERO,
            background: Color::from_rgb_u32(0xFCFCFD), // Tej Gray 25
            drag_over_background: Color::from_rgb_u32(0xF9F5FF), // Tej Brand 50
            border_color: Color::from_rgb_u32(0xD0D5DD), // Tej Gray 300
            corner_radius: 12.0,
        }
    }
}
