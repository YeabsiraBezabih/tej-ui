//! Navigation and structural layout widgets showcase view.

use std::collections::HashSet;
use tej_ui::prelude::*;

/// Category view showcasing breadcrumbs, pagination, accordions, and structural separators.
pub fn navigation_layout_view() -> impl View {
    let current_page = Signal::new(1usize);
    let open_accordion = Signal::new(HashSet::new());

    let breadcrumbs = vec![
        BreadcrumbItem::new("Home").href("/"),
        BreadcrumbItem::new("Workspace").href("/workspace"),
        BreadcrumbItem::new("Components").href("/components"),
        BreadcrumbItem::new("Storybook").mark_current(),
    ];

    let accordion_items = vec![
        AccordionItem::new(
            "sec1",
            "What makes Tej UI zero-copy?",
            "Tej UI binds directly to database WAL snapshots and memory-mapped buffers without heap cloning.",
        ),
        AccordionItem::new(
            "sec2",
            "Which native OS shells are supported?",
            "Wayland/X11 on Linux, DirectComposition on Windows, Metal on macOS, and WebGPU on Web.",
        ),
        AccordionItem::new(
            "sec3",
            "How does reactive scheduling avoid glitches?",
            "A topological generational slotmap arena ensures diamond dependencies evaluate without intermediate stale frames.",
        ),
    ];

    VStack::new((
        // Section 1: Breadcrumb Navigation
        Card::new("Breadcrumb Navigation Trail")
            .description("Hierarchical path indicators aiding spatial orientation")
            .flex_grow(1.0)
            .content(Breadcrumb::new(breadcrumbs)),
        // Section 2: Pagination Controls
        Card::new("Dataset Pagination")
            .description("Page navigation with previous/next bounds checking")
            .flex_grow(1.0)
            .content(
                HStack::new((
                    Pagination::new(current_page, 10),
                    Badge::new("10 PAGES TOTAL").variant(BadgeVariant::Secondary),
                ))
                .spacing(16),
            ),
        // Section 3: Collapsible Accordion Panels
        Card::new("Collapsible Accordion Panels")
            .description("Expandable disclosure containers revealing progressive detail")
            .flex_grow(1.0)
            .content(Accordion::new(open_accordion, accordion_items)),
    ))
    .spacing(16)
}
