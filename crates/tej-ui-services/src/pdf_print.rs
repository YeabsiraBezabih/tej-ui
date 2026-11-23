//! Print pagination pass and PDF vector document export pipeline.

use tej_ui_layout::{Rect, Size};

/// Standard paper sizes in typographical points (72 points = 1 inch).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PaperSize {
    A4,
    Letter,
    Legal,
    Custom { width_pt: f32, height_pt: f32 },
}

impl PaperSize {
    pub fn dimensions_points(&self) -> Size {
        match *self {
            PaperSize::A4 => Size::new(595.28, 841.89),
            PaperSize::Letter => Size::new(612.0, 792.0),
            PaperSize::Legal => Size::new(612.0, 1008.0),
            PaperSize::Custom {
                width_pt,
                height_pt,
            } => Size::new(width_pt, height_pt),
        }
    }
}

/// Page margin in points.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PageMargins {
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
    pub left: f32,
}

impl PageMargins {
    pub const STANDARD_HALF_INCH: Self = Self {
        top: 36.0,
        right: 36.0,
        bottom: 36.0,
        left: 36.0,
    };
    pub const ONE_INCH: Self = Self {
        top: 72.0,
        right: 72.0,
        bottom: 72.0,
        left: 72.0,
    };
}

/// Page layout parameters for print and PDF generation.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PageLayout {
    pub paper: PaperSize,
    pub margins: PageMargins,
}

impl PageLayout {
    pub fn new(paper: PaperSize, margins: PageMargins) -> Self {
        Self { paper, margins }
    }

    /// Computes the usable printable content rectangle.
    pub fn printable_area(&self) -> Rect {
        let dim = self.paper.dimensions_points();
        Rect::new(
            self.margins.left,
            self.margins.top,
            dim.width - self.margins.left - self.margins.right,
            dim.height - self.margins.top - self.margins.bottom,
        )
    }
}

impl Default for PageLayout {
    fn default() -> Self {
        Self::new(PaperSize::A4, PageMargins::STANDARD_HALF_INCH)
    }
}

/// Represents a single paginated page ready for spooling.
#[derive(Debug, Clone)]
pub struct PaginatedPage {
    pub page_number: usize,
    pub content_y_offset: f32,
    pub content_height: f32,
}

/// Pagination layout pass computing page count and vertical slice offsets.
#[derive(Debug, Clone)]
pub struct PdfPrintSpooler {
    layout: PageLayout,
}

impl PdfPrintSpooler {
    pub fn new(layout: PageLayout) -> Self {
        Self { layout }
    }

    pub fn layout(&self) -> PageLayout {
        self.layout
    }

    /// Paginates total vertical content height across discrete pages.
    pub fn paginate(&self, total_content_height: f32) -> Vec<PaginatedPage> {
        let page_h = self.layout.printable_area().height();
        if total_content_height <= 0.0 || page_h <= 0.0 {
            return vec![PaginatedPage {
                page_number: 1,
                content_y_offset: 0.0,
                content_height: 0.0,
            }];
        }

        let mut pages = Vec::new();
        let mut y = 0.0;
        let mut page_num = 1;

        while y < total_content_height {
            let slice_h = (total_content_height - y).min(page_h);
            pages.push(PaginatedPage {
                page_number: page_num,
                content_y_offset: y,
                content_height: slice_h,
            });
            y += slice_h;
            page_num += 1;
        }

        pages
    }
}
