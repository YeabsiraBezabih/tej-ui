//! Prompt Composer, Model Selector, and Attachment Tray for AI interfaces.
//! Styled with Tej's multi-line input borders, brand focus rings, and pill tags.

use tej_ui_hal::Color;
use tej_ui_layout::Rect;

/// AI Model selector option.
#[derive(Debug, Clone, PartialEq)]
pub struct ModelChoice {
    pub id: String,
    pub name: String,
    pub provider: String,
    pub context_limit_tokens: usize,
}

/// State for the multi-modal prompt composer.
#[derive(Debug, Clone, PartialEq)]
pub struct ComposerState {
    pub prompt_text: String,
    pub selected_model_id: String,
    pub attachments: Vec<String>,
    pub suggestion_pills: Vec<String>,
    pub is_generating: bool,
}

impl Default for ComposerState {
    fn default() -> Self {
        Self {
            prompt_text: String::new(),
            selected_model_id: "gemini-3.8-flash".to_string(),
            attachments: Vec::new(),
            suggestion_pills: Vec::new(),
            is_generating: false,
        }
    }
}

impl ComposerState {
    pub fn attach_file(&mut self, filename: impl Into<String>) {
        self.attachments.push(filename.into());
    }

    pub fn select_model(&mut self, model_id: impl Into<String>) {
        self.selected_model_id = model_id.into();
    }
}

/// Tej styled Prompt Composer component.
#[derive(Debug, Clone, PartialEq)]
pub struct PromptComposer {
    pub bounds: Rect,
    pub background: Color,
    pub border_color: Color,
    pub focus_ring_color: Color,
    pub corner_radius: f32,
}

impl Default for PromptComposer {
    fn default() -> Self {
        Self {
            bounds: Rect::ZERO,
            background: Color::WHITE,
            border_color: Color::from_rgb_u32(0xD0D5DD), // Tej Gray 300
            focus_ring_color: Color::from_rgb_u32(0x7F56D9), // Tej Brand 600
            corner_radius: 12.0,
        }
    }
}
