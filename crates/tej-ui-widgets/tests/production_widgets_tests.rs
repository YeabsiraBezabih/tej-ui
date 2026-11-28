//! Comprehensive unit tests verifying layout bounds, painting onto a recording surface,
//! keyboard accessibility, and collision flipping across all upgraded `tej-ui-widgets`.

use std::collections::HashSet;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use tej_ui_core::Signal;
use tej_ui_hal::{Color, MouseButton, SurfaceCompositor, UiEvent};
use tej_ui_layout::{Dimension, LayoutArena, LayoutEngine, Point, Rect, Size};
use tej_ui_view::{EventDispatcher, PaintContext, Text, Theme, View};
use tej_ui_widgets::{
    Accordion, AccordionItem, Avatar, Breadcrumb, BreadcrumbItem, CommandItem, CommandPalette,
    Pagination, Popover, PopoverPlacement, Progress, RadioGroup, RadioItem, RichTextEditor,
    ScrollArea, Select, SelectOption, Sheet, SheetSide, Skeleton, Switch, TabItem, Tabs, Tooltip,
    TreeNode, TreeView,
};

#[derive(Debug, Default)]
struct RecordingCompositor {
    filled_rects: Vec<(Rect, Color)>,
    filled_rounded_rects: Vec<(Rect, f32, Color)>,
    drawn_texts: Vec<(String, Point, f32, Color)>,
    drawn_lines: Vec<(Point, Point, f32, Color)>,
}

impl RecordingCompositor {
    fn all_rects(&self) -> Vec<Rect> {
        let mut rects: Vec<Rect> = self.filled_rects.iter().map(|(r, _)| *r).collect();
        rects.extend(self.filled_rounded_rects.iter().map(|(r, _, _)| *r));
        rects
    }

    fn has_text(&self, needle: &str) -> bool {
        self.drawn_texts
            .iter()
            .any(|(t, _, _, _)| t.contains(needle))
    }
}

impl SurfaceCompositor for RecordingCompositor {
    fn begin_frame(&mut self, _dirty_rects: &[Rect]) {}

    fn fill_rect(&mut self, rect: Rect, color: Color) {
        self.filled_rects.push((rect, color));
    }

    fn fill_rounded_rect(&mut self, rect: Rect, radius: f32, color: Color) {
        self.filled_rounded_rects.push((rect, radius, color));
    }

    fn draw_line(&mut self, p1: Point, p2: Point, stroke_width: f32, color: Color) {
        self.drawn_lines.push((p1, p2, stroke_width, color));
    }

    fn draw_text(&mut self, text: &str, origin: Point, font_size: f32, color: Color) {
        self.drawn_texts
            .push((text.to_string(), origin, font_size, color));
    }

    fn end_frame(&mut self) {}
}

fn click_node(dispatcher: &mut EventDispatcher, arena: &LayoutArena, root: u32, node: u32) {
    let bounds = arena.computed_bounds[node as usize];
    let center = Point::new(
        bounds.origin.x + bounds.size.width * 0.5,
        bounds.origin.y + bounds.size.height * 0.5,
    );
    dispatcher.dispatch(
        &UiEvent::MouseDown {
            button: MouseButton::Left,
            position: center,
        },
        arena,
        root,
    );
    dispatcher.dispatch(
        &UiEvent::MouseUp {
            button: MouseButton::Left,
            position: center,
        },
        arena,
        root,
    );
}

#[test]
fn test_command_palette_layout_paint_and_keyboard_click_nav() {
    let executed = Arc::new(AtomicUsize::new(0));
    let ex1 = Arc::clone(&executed);
    let ex2 = Arc::clone(&executed);

    let query = Signal::new(String::new());
    let selected = Signal::new(0usize);
    let is_open = Signal::new(true);

    let items = vec![
        CommandItem::new("1", "Open Settings", "Preferences")
            .shortcut("Cmd+,")
            .on_select(move || {
                ex1.store(1, Ordering::SeqCst);
            }),
        CommandItem::new("2", "Reset Cache", "Developer")
            .shortcut("Cmd+R")
            .on_select(move || {
                ex2.store(2, Ordering::SeqCst);
            }),
        CommandItem::new("3", "Toggle Terminal", "View"),
    ];

    let palette = CommandPalette::new(query, selected, items).open(is_open);
    let mut state = palette.build();
    let mut arena = LayoutArena::new();
    let root = palette.layout(&mut state, &mut arena);
    LayoutEngine::compute_layout(&mut arena, root, Size::new(800.0, 600.0));

    let root_bounds = arena.computed_bounds[state.node as usize];
    assert_eq!(root_bounds.size.width, 520.0);
    assert!(root_bounds.size.height > 100.0);
    assert_eq!(state.item_nodes.len(), 3);

    // Paint and verify search box, items, shortcut badges, and highlight
    let mut compositor = RecordingCompositor::default();
    let theme = Theme::dark();
    let dirty: [Rect; 0] = [];
    {
        let mut ctx = PaintContext::new(&mut compositor, &arena, &theme, &dirty);
        palette.paint(&state, &mut ctx);
    }
    assert!(compositor.has_text("Type a command or search..."));
    assert!(compositor.has_text("Open Settings"));
    assert!(compositor.has_text("Cmd+,"));
    assert!(compositor.has_text("Reset Cache"));
    assert_ne!(
        compositor.drawn_lines,
        [] as [(
            tej_ui_layout::Point,
            tej_ui_layout::Point,
            f32,
            tej_ui_hal::Color
        ); 0]
    );

    // Keyboard navigation: Down -> index 1, Up -> index 0, Down -> index 1, Enter -> executes item 2
    let mut dispatcher = EventDispatcher::new();
    palette.register_events(&state, &mut dispatcher);
    dispatcher.focus(Some(state.search_node));

    dispatcher.dispatch(&UiEvent::KeyDown { key_code: 40 }, &arena, root);
    assert_eq!(selected.get(), 1);
    dispatcher.dispatch(&UiEvent::KeyDown { key_code: 38 }, &arena, root);
    assert_eq!(selected.get(), 0);
    dispatcher.dispatch(&UiEvent::KeyDown { key_code: 40 }, &arena, root);
    assert_eq!(selected.get(), 1);
    dispatcher.dispatch(&UiEvent::KeyDown { key_code: 13 }, &arena, root);
    assert_eq!(executed.load(Ordering::SeqCst), 2);
    assert!(!is_open.get());

    // Reopen, test click selection on row 0 and Escape dismissal
    is_open.set(true);
    palette.rebuild(&mut state);
    let mut arena2 = LayoutArena::new();
    let root2 = palette.layout(&mut state, &mut arena2);
    LayoutEngine::compute_layout(&mut arena2, root2, Size::new(800.0, 600.0));
    let mut dispatcher2 = EventDispatcher::new();
    palette.register_events(&state, &mut dispatcher2);

    click_node(&mut dispatcher2, &arena2, root2, state.item_nodes[0]);
    assert_eq!(executed.load(Ordering::SeqCst), 1);
    assert_eq!(selected.get(), 0);
    assert!(!is_open.get());

    // Test Escape clears query and closes
    is_open.set(true);
    query.set("set".to_string());
    palette.rebuild(&mut state);
    let mut arena3 = LayoutArena::new();
    let root3 = palette.layout(&mut state, &mut arena3);
    LayoutEngine::compute_layout(&mut arena3, root3, Size::new(800.0, 600.0));
    let mut dispatcher3 = EventDispatcher::new();
    palette.register_events(&state, &mut dispatcher3);
    dispatcher3.focus(Some(state.node));

    dispatcher3.dispatch(&UiEvent::KeyDown { key_code: 27 }, &arena3, root3);
    assert_eq!(query.get(), "");
    assert!(!is_open.get());
}

#[test]
fn test_feedback_widgets_avatar_progress_skeleton_tooltip_layout_and_paint() {
    let theme = Theme::dark();
    let dirty: [Rect; 0] = [];

    // 1. Avatar
    let avatar = Avatar::new("Ada Lovelace").diameter(48.0);
    let mut av_state = avatar.build();
    let mut arena = LayoutArena::new();
    let av_root = avatar.layout(&mut av_state, &mut arena);
    LayoutEngine::compute_layout(&mut arena, av_root, Size::new(200.0, 200.0));
    assert_eq!(
        arena.computed_bounds[av_state.node as usize].size,
        Size::new(48.0, 48.0)
    );

    let mut comp = RecordingCompositor::default();
    {
        let mut ctx = PaintContext::new(&mut comp, &arena, &theme, &dirty);
        avatar.paint(&av_state, &mut ctx);
    }
    assert!(
        comp.filled_rounded_rects
            .iter()
            .any(|(_, r, _)| (*r - 24.0).abs() < 0.01)
    );
    assert!(comp.has_text("AL"));

    // 2. Progress
    let pct = Signal::new(50.0f32);
    let progress = Progress::new(pct).height(10.0);
    let mut pg_state = progress.build();
    let mut arena = LayoutArena::new();
    let pg_root = progress.layout(&mut pg_state, &mut arena);
    LayoutEngine::compute_layout(&mut arena, pg_root, Size::new(200.0, 40.0));
    assert_eq!(
        arena.computed_bounds[pg_state.node as usize].size,
        Size::new(200.0, 10.0)
    );

    let mut comp = RecordingCompositor::default();
    {
        let mut ctx = PaintContext::new(&mut comp, &arena, &theme, &dirty);
        progress.paint(&pg_state, &mut ctx);
    }
    assert_eq!(comp.filled_rounded_rects.len(), 2);
    assert!((comp.filled_rounded_rects[0].0.size.width - 200.0).abs() < 0.01);
    assert!((comp.filled_rounded_rects[1].0.size.width - 100.0).abs() < 0.01);

    // 3. Skeleton
    let skeleton = Skeleton::new(Dimension::Points(160.0), Dimension::Points(24.0));
    let mut sk_state = skeleton.build();
    let mut arena = LayoutArena::new();
    let sk_root = skeleton.layout(&mut sk_state, &mut arena);
    LayoutEngine::compute_layout(&mut arena, sk_root, Size::new(300.0, 100.0));
    assert_eq!(
        arena.computed_bounds[sk_state.node as usize].size,
        Size::new(160.0, 24.0)
    );

    let mut comp = RecordingCompositor::default();
    {
        let mut ctx = PaintContext::new(&mut comp, &arena, &theme, &dirty);
        skeleton.paint(&sk_state, &mut ctx);
    }
    // Base block + shimmer stripe
    assert_eq!(comp.filled_rounded_rects.len(), 2);

    // 4. Tooltip (hidden -> paints nothing; visible -> paints dark pill + text)
    let visible = Signal::new(false);
    let tooltip = Tooltip::new("Save changes", visible);
    let mut tt_state = tooltip.build();
    let mut arena = LayoutArena::new();
    let tt_root = tooltip.layout(&mut tt_state, &mut arena);
    LayoutEngine::compute_layout(&mut arena, tt_root, Size::new(300.0, 100.0));
    assert_eq!(
        arena.computed_bounds[tt_state.node as usize].size,
        Size::new(0.0, 0.0)
    );

    let mut comp_hidden = RecordingCompositor::default();
    {
        let mut ctx = PaintContext::new(&mut comp_hidden, &arena, &theme, &dirty);
        tooltip.paint(&tt_state, &mut ctx);
    }
    assert_eq!(comp_hidden.all_rects(), [] as [tej_ui_layout::Rect; 0]);
    assert_eq!(
        comp_hidden.drawn_texts,
        [] as [(
            std::string::String,
            tej_ui_layout::Point,
            f32,
            tej_ui_hal::Color
        ); 0]
    );

    visible.set(true);
    tooltip.rebuild(&mut tt_state);
    let mut arena = LayoutArena::new();
    let tt_root = tooltip.layout(&mut tt_state, &mut arena);
    LayoutEngine::compute_layout(&mut arena, tt_root, Size::new(300.0, 100.0));
    assert!(arena.computed_bounds[tt_state.node as usize].size.width > 80.0);

    let mut comp_vis = RecordingCompositor::default();
    {
        let mut ctx = PaintContext::new(&mut comp_vis, &arena, &theme, &dirty);
        tooltip.paint(&tt_state, &mut ctx);
    }
    assert!(comp_vis.has_text("Save changes"));
    assert!(
        comp_vis
            .filled_rounded_rects
            .iter()
            .any(|(_, _, c)| *c == Color::from_rgb_u32(0x0018_181B))
    );
}

#[test]
fn test_accordion_layout_paint_click_and_keyboard_toggle() {
    let open = Signal::new(HashSet::new());
    let accordion = Accordion::new(
        open,
        vec![
            AccordionItem::new("faq1", "Billing", "Monthly invoicing details"),
            AccordionItem::new("faq2", "Security", "SOC2 Type II certified"),
        ],
    );

    let mut state = accordion.build();
    let mut arena = LayoutArena::new();
    let root = accordion.layout(&mut state, &mut arena);
    LayoutEngine::compute_layout(&mut arena, root, Size::new(400.0, 400.0));

    let mut dispatcher = EventDispatcher::new();
    accordion.register_events(&state, &mut dispatcher);

    // Click first header expands faq1
    click_node(&mut dispatcher, &arena, root, state.header_nodes[0]);
    assert!(open.get().contains("faq1"));

    // Rebuild, re-layout, and paint to verify content panel is rendered
    accordion.rebuild(&mut state);
    let mut arena2 = LayoutArena::new();
    let root2 = accordion.layout(&mut state, &mut arena2);
    LayoutEngine::compute_layout(&mut arena2, root2, Size::new(400.0, 400.0));
    assert!(state.content_nodes[0].is_some());

    let mut comp = RecordingCompositor::default();
    let theme = Theme::dark();
    let dirty: [Rect; 0] = [];
    {
        let mut ctx = PaintContext::new(&mut comp, &arena2, &theme, &dirty);
        accordion.paint(&state, &mut ctx);
    }
    assert!(comp.has_text("Billing"));
    assert!(comp.has_text("Monthly invoicing details"));
    assert!(comp.has_text("Security"));

    // Space (32) collapses focused header 0, Enter (13) expands header 1
    let mut dispatcher2 = EventDispatcher::new();
    accordion.register_events(&state, &mut dispatcher2);
    dispatcher2.focus(Some(state.header_nodes[0]));
    dispatcher2.dispatch(&UiEvent::KeyDown { key_code: 32 }, &arena2, root2);
    assert!(!open.get().contains("faq1"));

    dispatcher2.focus(Some(state.header_nodes[1]));
    dispatcher2.dispatch(&UiEvent::KeyDown { key_code: 13 }, &arena2, root2);
    assert!(open.get().contains("faq2"));
}

#[test]
fn test_scroll_area_viewport_clipping_and_scrollbar_thumbs() {
    let sx = Signal::new(0.0f32);
    let sy = Signal::new(200.0f32);
    let scroll = ScrollArea::new(sx, sy, Size::new(800.0, 1600.0), Size::new(300.0, 400.0));

    let mut state = scroll.build();
    let mut arena = LayoutArena::new();
    let root = scroll.layout(&mut state, &mut arena);
    LayoutEngine::compute_layout(&mut arena, root, Size::new(600.0, 600.0));

    let vp = arena.computed_bounds[state.node as usize];
    assert_eq!(vp.size, Size::new(300.0, 400.0));

    let mut comp = RecordingCompositor::default();
    let theme = Theme::dark();
    let dirty: [Rect; 0] = [];
    {
        let mut ctx = PaintContext::new(&mut comp, &arena, &theme, &dirty);
        scroll.paint(&state, &mut ctx);
    }

    // All painted rectangles must stay strictly clipped inside viewport bounds
    assert_ne!(comp.all_rects(), [] as [tej_ui_layout::Rect; 0]);
    for rect in comp.all_rects() {
        assert!(rect.origin.x >= vp.origin.x - 0.01);
        assert!(rect.origin.y >= vp.origin.y - 0.01);
        assert!(rect.origin.x + rect.size.width <= vp.origin.x + vp.size.width + 0.01);
        assert!(rect.origin.y + rect.size.height <= vp.origin.y + vp.size.height + 0.01);
    }

    // Both vertical and horizontal scrollbar thumbs are computed and painted
    assert!(scroll.vertical_thumb_rect(vp, sy.get()).is_some());
    assert!(scroll.horizontal_thumb_rect(vp, sx.get()).is_some());

    // Keyboard ArrowDown scrolls vertically
    let mut dispatcher = EventDispatcher::new();
    scroll.register_events(&state, &mut dispatcher);
    dispatcher.focus(Some(state.node));
    dispatcher.dispatch(&UiEvent::KeyDown { key_code: 40 }, &arena, root);
    assert_eq!(sy.get(), 240.0);
}

#[test]
fn test_breadcrumb_pagination_and_sheet_layout_paint_and_events() {
    let theme = Theme::dark();
    let dirty: [Rect; 0] = [];

    // 1. Breadcrumb
    let nav_called = Arc::new(AtomicBool::new(false));
    let nav_flag = Arc::clone(&nav_called);
    let bc = Breadcrumb::new(vec![
        BreadcrumbItem::new("Home").href("/home"),
        BreadcrumbItem::new("Projects").href("/projects"),
        BreadcrumbItem::new("Settings").mark_current(),
    ])
    .separator(">")
    .on_navigate(move |href| {
        if href == "/home" {
            nav_flag.store(true, Ordering::SeqCst);
        }
    });

    let mut bc_state = bc.build();
    let mut arena = LayoutArena::new();
    let bc_root = bc.layout(&mut bc_state, &mut arena);
    LayoutEngine::compute_layout(&mut arena, bc_root, Size::new(500.0, 40.0));
    assert_eq!(bc_state.item_nodes.len(), 3);

    let mut comp = RecordingCompositor::default();
    {
        let mut ctx = PaintContext::new(&mut comp, &arena, &theme, &dirty);
        bc.paint(&bc_state, &mut ctx);
    }
    assert!(comp.has_text("Home"));
    assert!(comp.has_text(">"));
    assert!(comp.has_text("Settings"));

    let mut dispatcher = EventDispatcher::new();
    bc.register_events(&bc_state, &mut dispatcher);
    click_node(&mut dispatcher, &arena, bc_root, bc_state.item_nodes[0]);
    assert!(nav_called.load(Ordering::SeqCst));

    // 2. Pagination
    let page = Signal::new(1usize);
    let pg = Pagination::new(page, 5);
    let mut pg_state = pg.build();
    let mut arena = LayoutArena::new();
    let pg_root = pg.layout(&mut pg_state, &mut arena);
    LayoutEngine::compute_layout(&mut arena, pg_root, Size::new(500.0, 50.0));

    let mut comp = RecordingCompositor::default();
    {
        let mut ctx = PaintContext::new(&mut comp, &arena, &theme, &dirty);
        pg.paint(&pg_state, &mut ctx);
    }
    assert!(comp.has_text("Prev"));
    assert!(comp.has_text("Next"));
    assert!(comp.has_text("1"));
    assert!(comp.has_text("5"));

    let mut dispatcher = EventDispatcher::new();
    pg.register_events(&pg_state, &mut dispatcher);
    // Click Next -> page 2
    click_node(&mut dispatcher, &arena, pg_root, pg_state.next_node);
    assert_eq!(page.get(), 2);
    // Click page 4 directly
    click_node(&mut dispatcher, &arena, pg_root, pg_state.page_nodes[3].1);
    assert_eq!(page.get(), 4);
    // ArrowLeft -> page 3, ArrowRight -> page 4
    dispatcher.focus(Some(pg_state.node));
    dispatcher.dispatch(&UiEvent::KeyDown { key_code: 37 }, &arena, pg_root);
    assert_eq!(page.get(), 3);
    dispatcher.dispatch(&UiEvent::KeyDown { key_code: 39 }, &arena, pg_root);
    assert_eq!(page.get(), 4);

    // 3. Sheet
    let sheet_open = Signal::new(true);
    let sheet = Sheet::new(sheet_open, "Inspector")
        .side(SheetSide::Right)
        .size(320.0);
    let mut sh_state = sheet.build();
    let mut arena = LayoutArena::new();
    let sh_root = sheet.layout(&mut sh_state, &mut arena);
    LayoutEngine::compute_layout(&mut arena, sh_root, Size::new(1000.0, 700.0));

    let panel_bounds = arena.computed_bounds[sh_state.panel_node as usize];
    assert_eq!(panel_bounds.size.width, 320.0);
    assert_eq!(panel_bounds.size.height, 700.0);
    assert!((panel_bounds.origin.x - 680.0).abs() < 0.5);

    let mut comp = RecordingCompositor::default();
    {
        let mut ctx = PaintContext::new(&mut comp, &arena, &theme, &dirty);
        sheet.paint(&sh_state, &mut ctx);
    }
    assert!(comp.has_text("Inspector"));
    assert!(
        comp.filled_rects
            .iter()
            .any(|(_, c)| *c == Color::rgba(0, 0, 0, 140))
    );

    let mut dispatcher = EventDispatcher::new();
    sheet.register_events(&sh_state, &mut dispatcher);
    dispatcher.focus(Some(sh_state.panel_node));
    dispatcher.dispatch(&UiEvent::KeyDown { key_code: 27 }, &arena, sh_root);
    assert!(!sheet_open.get());
}

#[test]
fn test_keyboard_accessibility_switch_radio_select_tabs() {
    // 1. Switch: Space (32), Enter (13), ArrowRight (39), ArrowLeft (37)
    let sw_sig = Signal::new(false);
    let sw = Switch::new(sw_sig);
    let mut sw_state = sw.build();
    let mut arena = LayoutArena::new();
    let root = sw.layout(&mut sw_state, &mut arena);
    LayoutEngine::compute_layout(&mut arena, root, Size::new(200.0, 50.0));
    let mut dispatcher = EventDispatcher::new();
    sw.register_events(&sw_state, &mut dispatcher);
    dispatcher.focus(Some(sw_state.node));

    dispatcher.dispatch(&UiEvent::KeyDown { key_code: 32 }, &arena, root);
    assert!(sw_sig.get());
    dispatcher.dispatch(&UiEvent::KeyDown { key_code: 13 }, &arena, root);
    assert!(!sw_sig.get());
    dispatcher.dispatch(&UiEvent::KeyDown { key_code: 39 }, &arena, root);
    assert!(sw_sig.get());
    dispatcher.dispatch(&UiEvent::KeyDown { key_code: 37 }, &arena, root);
    assert!(!sw_sig.get());

    // 2. RadioGroup: ArrowDown (40), ArrowUp (38)
    let radio_sig = Signal::new("a".to_string());
    let radio = RadioGroup::new(
        radio_sig,
        vec![
            RadioItem::new("a".to_string(), "Alpha"),
            RadioItem::new("b".to_string(), "Beta"),
            RadioItem::new("c".to_string(), "Gamma"),
        ],
    );
    let mut rg_state = radio.build();
    let mut arena = LayoutArena::new();
    let root = radio.layout(&mut rg_state, &mut arena);
    LayoutEngine::compute_layout(&mut arena, root, Size::new(200.0, 120.0));
    let mut dispatcher = EventDispatcher::new();
    radio.register_events(&rg_state, &mut dispatcher);
    dispatcher.focus(Some(rg_state.item_nodes[0]));

    dispatcher.dispatch(&UiEvent::KeyDown { key_code: 40 }, &arena, root);
    assert_eq!(radio_sig.get(), "b");
    dispatcher.dispatch(&UiEvent::KeyDown { key_code: 40 }, &arena, root);
    assert_eq!(radio_sig.get(), "c");
    dispatcher.dispatch(&UiEvent::KeyDown { key_code: 38 }, &arena, root);
    assert_eq!(radio_sig.get(), "b");

    // 3. Select: Space (32) opens, ArrowDown (40) cycles, Escape (27) closes
    let sel_sig = Signal::new("usd".to_string());
    let sel_open = Signal::new(false);
    let select = Select::new(
        sel_sig,
        sel_open,
        vec![
            SelectOption::new("usd".to_string(), "USD"),
            SelectOption::new("eur".to_string(), "EUR"),
            SelectOption::new("etb".to_string(), "ETB"),
        ],
        "Currency",
    );
    let mut sel_state = select.build();
    let mut arena = LayoutArena::new();
    let root = select.layout(&mut sel_state, &mut arena);
    LayoutEngine::compute_layout(&mut arena, root, Size::new(240.0, 200.0));
    let mut dispatcher = EventDispatcher::new();
    select.register_events(&sel_state, &mut dispatcher);
    dispatcher.focus(Some(sel_state.node));

    dispatcher.dispatch(&UiEvent::KeyDown { key_code: 32 }, &arena, root);
    assert!(sel_open.get());
    dispatcher.dispatch(&UiEvent::KeyDown { key_code: 40 }, &arena, root);
    assert_eq!(sel_sig.get(), "eur");
    dispatcher.dispatch(&UiEvent::KeyDown { key_code: 27 }, &arena, root);
    assert!(!sel_open.get());

    // 4. Tabs: ArrowRight (39), ArrowLeft (37), Enter (13)
    let active_tab = Signal::new(0usize);
    let tabs = Tabs::new(
        vec![
            TabItem::new("t1", "General"),
            TabItem::new("t2", "Billing"),
            TabItem::new("t3", "API"),
        ],
        active_tab,
        |idx| Text::new(format!("Panel {idx}")),
    );
    let mut tab_state = tabs.build();
    let mut arena = LayoutArena::new();
    let root = tabs.layout(&mut tab_state, &mut arena);
    LayoutEngine::compute_layout(&mut arena, root, Size::new(400.0, 200.0));
    let mut dispatcher = EventDispatcher::new();
    tabs.register_events(&tab_state, &mut dispatcher);
    dispatcher.focus(Some(tab_state.tab_nodes[0]));

    dispatcher.dispatch(&UiEvent::KeyDown { key_code: 39 }, &arena, root);
    assert_eq!(active_tab.get(), 1);
    dispatcher.dispatch(&UiEvent::KeyDown { key_code: 39 }, &arena, root);
    assert_eq!(active_tab.get(), 2);
    dispatcher.dispatch(&UiEvent::KeyDown { key_code: 37 }, &arena, root);
    assert_eq!(active_tab.get(), 1);
}

#[test]
fn test_tree_view_expand_collapse_on_click_and_arrows() {
    let toggle_count = Arc::new(AtomicUsize::new(0));
    let tc = Arc::clone(&toggle_count);

    let roots = vec![
        TreeNode::branch(
            "src",
            "src",
            vec![
                TreeNode::leaf("main.rs", "main.rs"),
                TreeNode::leaf("lib.rs", "lib.rs"),
            ],
        ),
        TreeNode::leaf("Cargo.toml", "Cargo.toml"),
    ];
    let selected = Signal::new(None);
    let tree = TreeView::new(roots, selected).on_toggle(move |_id, _expanded| {
        tc.fetch_add(1, Ordering::SeqCst);
    });

    assert_eq!(tree.visible_flattened().len(), 4);
    assert!(tree.is_expanded("src"));

    let mut state = tree.build();
    let mut arena = LayoutArena::new();
    let root = tree.layout(&mut state, &mut arena);
    LayoutEngine::compute_layout(&mut arena, root, Size::new(300.0, 200.0));
    let mut dispatcher = EventDispatcher::new();
    tree.register_events(&state, &mut dispatcher);

    // Click branch "src" -> collapses it
    click_node(&mut dispatcher, &arena, root, state.item_nodes[0]);
    assert_eq!(selected.get().as_deref(), Some("src"));
    assert!(!tree.is_expanded("src"));
    assert_eq!(tree.visible_flattened().len(), 2);
    assert_eq!(toggle_count.load(Ordering::SeqCst), 1);

    // ArrowRight (39) on focused "src" -> expands it
    dispatcher.focus(Some(state.item_nodes[0]));
    dispatcher.dispatch(&UiEvent::KeyDown { key_code: 39 }, &arena, root);
    assert!(tree.is_expanded("src"));
    assert_eq!(tree.visible_flattened().len(), 4);
    assert_eq!(toggle_count.load(Ordering::SeqCst), 2);

    // ArrowLeft (37) on focused "src" -> collapses it again
    dispatcher.dispatch(&UiEvent::KeyDown { key_code: 37 }, &arena, root);
    assert!(!tree.is_expanded("src"));
    assert_eq!(tree.visible_flattened().len(), 2);
}

#[test]
fn test_rich_text_editor_keyboard_events_and_character_input() {
    let editor = RichTextEditor::with_text("hi");
    let mut state = editor.build();
    let mut arena = LayoutArena::new();
    let root = editor.layout(&mut state, &mut arena);
    LayoutEngine::compute_layout(&mut arena, root, Size::new(400.0, 200.0));

    let mut dispatcher = EventDispatcher::new();
    editor.register_events(&state, &mut dispatcher);
    dispatcher.focus(Some(state.node));

    // Move right twice to end of "hi", type "!", press Enter, type "ok"
    dispatcher.dispatch(&UiEvent::KeyDown { key_code: 39 }, &arena, root);
    dispatcher.dispatch(&UiEvent::KeyDown { key_code: 39 }, &arena, root);
    dispatcher.dispatch(
        &UiEvent::TextInput {
            text: "!".to_string(),
        },
        &arena,
        root,
    );
    assert_eq!(editor.text(), "hi!");

    // Enter (13) inserts newline
    dispatcher.dispatch(&UiEvent::KeyDown { key_code: 13 }, &arena, root);
    dispatcher.dispatch(
        &UiEvent::TextInput {
            text: "ok".to_string(),
        },
        &arena,
        root,
    );
    assert_eq!(editor.text(), "hi!\nok");

    // Backspace (8) deletes 'k'
    dispatcher.dispatch(&UiEvent::KeyDown { key_code: 8 }, &arena, root);
    assert_eq!(editor.text(), "hi!\no");

    // ArrowUp (38) moves to line 0, ArrowLeft (37) moves to col 0, Delete (46) deletes 'h'
    dispatcher.dispatch(&UiEvent::KeyDown { key_code: 38 }, &arena, root);
    dispatcher.dispatch(&UiEvent::KeyDown { key_code: 37 }, &arena, root);
    dispatcher.dispatch(&UiEvent::KeyDown { key_code: 46 }, &arena, root);
    assert_eq!(editor.text(), "i!\no");
}

#[test]
fn test_popover_collision_flipping_in_layout_and_paint() {
    let viewport = Rect::new(0.0, 0.0, 800.0, 600.0);
    // Anchor near bottom edge: preferred Bottom (120px tall) overflows 600px -> flips to Top!
    let bottom_anchor = Rect::new(300.0, 540.0, 100.0, 36.0);
    let is_open = Signal::new(true);

    let popover = Popover::new(is_open, Text::new("Trigger"), Text::new("Menu"))
        .placement(PopoverPlacement::Bottom)
        .viewport(viewport)
        .anchor_bounds(bottom_anchor)
        .overlay_size(Size::new(200.0, 120.0));

    let mut state = popover.build();
    let mut arena = LayoutArena::new();
    let root = popover.layout(&mut state, &mut arena);
    LayoutEngine::compute_layout(&mut arena, root, Size::new(800.0, 600.0));

    assert!(state.is_flipped);
    assert_eq!(state.resolved_placement, PopoverPlacement::Top);
    let flipped_rect = state.content_rect.expect("collision rect computed");
    assert!(flipped_rect.origin.y + flipped_rect.size.height <= bottom_anchor.origin.y);

    // Paint and verify the floating popover surface is drawn at the flipped Top coordinates
    let mut comp = RecordingCompositor::default();
    let theme = Theme::dark();
    let dirty: [Rect; 0] = [];
    {
        let mut ctx = PaintContext::new(&mut comp, &arena, &theme, &dirty);
        popover.paint(&state, &mut ctx);
    }
    assert!(
        comp.filled_rounded_rects
            .iter()
            .any(|(r, _, _)| (r.origin.y - flipped_rect.origin.y).abs() < 0.5
                && (r.size.height - 120.0).abs() < 0.5)
    );

    // Escape key closes popover
    let mut dispatcher = EventDispatcher::new();
    popover.register_events(&state, &mut dispatcher);
    dispatcher.focus(Some(state.node));
    dispatcher.dispatch(&UiEvent::KeyDown { key_code: 27 }, &arena, root);
    assert!(!is_open.get());
}
