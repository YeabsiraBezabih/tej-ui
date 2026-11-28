//! Phase 4 visual, layout, clipping, and default styling verification tests for `tej-ui-widgets`.

use tej_ui_core::Signal;
use tej_ui_hal::{Color, SurfaceCompositor};
use tej_ui_layout::{Dimension, LayoutArena, LayoutEngine, Point, Rect, Size};
use tej_ui_text::FontRegistry;
use tej_ui_view::{PaintContext, Text, Theme, View};
use tej_ui_widgets::{
    Alert, AlertSeverity, Badge, BadgeVariant, Card, DataGrid, DataGridDimensions, GridColumn,
    Modal, Progress, RichTextEditor, Select, SelectOption, Sheet, SheetSide, Sidebar, SidebarItem,
    Slider, Switch, TabItem, Tabs, Tooltip,
};

#[derive(Debug, Default)]
struct ClipRecordingCompositor {
    filled_rects: Vec<(Rect, Color)>,
    filled_rounded_rects: Vec<(Rect, f32, Color)>,
    drawn_texts: Vec<(String, Point, f32, Color)>,
    pushed_clips: Vec<(Rect, [f32; 4])>,
    popped_clips: usize,
}

impl SurfaceCompositor for ClipRecordingCompositor {
    fn begin_frame(&mut self, _dirty_rects: &[Rect]) {}

    fn fill_rect(&mut self, rect: Rect, color: Color) {
        self.filled_rects.push((rect, color));
    }

    fn fill_rounded_rect(&mut self, rect: Rect, radius: f32, color: Color) {
        self.filled_rounded_rects.push((rect, radius, color));
    }

    fn draw_line(&mut self, _p1: Point, _p2: Point, _stroke_width: f32, _color: Color) {}

    fn draw_text(&mut self, text: &str, origin: Point, font_size: f32, color: Color) {
        self.drawn_texts
            .push((text.to_string(), origin, font_size, color));
    }

    fn push_clip(&mut self, rect: Rect, corner_radii: [f32; 4]) {
        self.pushed_clips.push((rect, corner_radii));
    }

    fn pop_clip(&mut self) {
        self.popped_clips += 1;
    }

    fn end_frame(&mut self) {}
}

#[test]
fn test_badge_measurement_centering_clipping_outline_and_theme_tokens() {
    let theme = Theme::light();
    let dirty: [Rect; 0] = [];

    // 1. Outline variant draws border pill + inner background cutout, measures text, centers, and clips
    let badge = Badge::new("Outline").variant(BadgeVariant::Outline);
    let mut state = badge.build();
    let mut arena = LayoutArena::new();
    let root = badge.layout(&mut state, &mut arena);
    LayoutEngine::compute_layout(&mut arena, root, Size::new(300.0, 100.0));

    let measured = FontRegistry::global().measure_text(
        "Outline",
        theme.typography.caption.font_size,
        tej_ui_text::FontFamilyKind::SansSerif,
        400,
    );
    let expected_w = measured.width.ceil() + 16.0;
    let bounds = arena.computed_bounds[state.node as usize];
    assert!((bounds.size.width - expected_w).abs() < 0.01);

    let mut comp = ClipRecordingCompositor::default();
    {
        let mut ctx = PaintContext::new(&mut comp, &arena, &theme, &dirty);
        badge.paint(&state, &mut ctx);
    }
    assert!(
        comp.filled_rounded_rects
            .iter()
            .any(|(_, _, c)| *c == theme.colors.border)
    );
    assert!(
        comp.filled_rounded_rects
            .iter()
            .any(|(_, _, c)| *c == theme.colors.background)
    );
    assert_eq!(comp.pushed_clips.len(), 1);
    assert_eq!(comp.popped_clips, 1);

    // Centered horizontally and vertically inside pill
    let (_, text_pt, _, _) = &comp.drawn_texts[0];
    let expected_x = bounds.origin.x + ((bounds.size.width - measured.width) * 0.5).max(4.0);
    let expected_y = bounds.origin.y + ((bounds.size.height - measured.height) * 0.5).max(0.0);
    assert!((text_pt.x - expected_x).abs() < 0.01);
    assert!((text_pt.y - expected_y).abs() < 0.01);

    // 2. Success and Warning variants use theme tokens
    for (variant, expected_bg, expected_fg) in [
        (
            BadgeVariant::Success,
            theme.colors.success,
            theme.colors.success_foreground,
        ),
        (
            BadgeVariant::Warning,
            theme.colors.warning,
            theme.colors.warning_foreground,
        ),
    ] {
        let b = Badge::new("Status").variant(variant);
        let mut st = b.build();
        let mut ar = LayoutArena::new();
        let r = b.layout(&mut st, &mut ar);
        LayoutEngine::compute_layout(&mut ar, r, Size::new(200.0, 50.0));
        let mut c = ClipRecordingCompositor::default();
        {
            let mut ctx = PaintContext::new(&mut c, &ar, &theme, &dirty);
            b.paint(&st, &mut ctx);
        }
        assert!(
            c.filled_rounded_rects
                .iter()
                .any(|(_, _, col)| *col == expected_bg)
        );
        assert!(
            c.drawn_texts
                .iter()
                .any(|(_, _, _, col)| *col == expected_fg)
        );
    }
}

#[test]
fn test_progress_width_builder_and_rounded_clipping() {
    let pct = Signal::new(35.0);
    let progress = Progress::new(pct)
        .width(Dimension::Points(160.0))
        .height(10.0);
    let mut state = progress.build();
    let mut arena = LayoutArena::new();
    let root = progress.layout(&mut state, &mut arena);
    LayoutEngine::compute_layout(&mut arena, root, Size::new(400.0, 100.0));

    let bounds = arena.computed_bounds[state.node as usize];
    assert!((bounds.size.width - 160.0).abs() < 0.01);
    assert!((bounds.size.height - 10.0).abs() < 0.01);

    let mut comp = ClipRecordingCompositor::default();
    let theme = Theme::dark();
    let dirty: [Rect; 0] = [];
    {
        let mut ctx = PaintContext::new(&mut comp, &arena, &theme, &dirty);
        progress.paint(&state, &mut ctx);
    }
    assert_eq!(comp.pushed_clips.len(), 1);
    assert_eq!(comp.popped_clips, 1);
    assert_eq!(comp.pushed_clips[0].0, bounds);
    assert_eq!(comp.pushed_clips[0].1, [5.0; 4]);
}

#[test]
fn test_alert_and_tooltip_theme_tokens_and_clipping() {
    let theme = Theme::light();
    let dirty: [Rect; 0] = [];

    // Alert uses theme info/success/warning/destructive tokens and clips text
    for (severity, expected_accent) in [
        (AlertSeverity::Info, theme.colors.info),
        (AlertSeverity::Success, theme.colors.success),
        (AlertSeverity::Warning, theme.colors.warning),
        (AlertSeverity::Destructive, theme.colors.destructive),
    ] {
        let alert = Alert::new("Notice")
            .description("Detailed description text")
            .severity(severity);
        let mut state = alert.build();
        let mut arena = LayoutArena::new();
        let root = alert.layout(&mut state, &mut arena);
        LayoutEngine::compute_layout(&mut arena, root, Size::new(500.0, 200.0));

        let mut comp = ClipRecordingCompositor::default();
        {
            let mut ctx = PaintContext::new(&mut comp, &arena, &theme, &dirty);
            alert.paint(&state, &mut ctx);
        }
        assert!(
            comp.filled_rects
                .iter()
                .any(|(_, col)| *col == expected_accent)
        );
        assert_eq!(comp.pushed_clips.len(), 1);
        assert_eq!(comp.popped_clips, 1);
    }

    // Tooltip uses theme popover/popover_foreground tokens and clips text
    let visible = Signal::new(true);
    let tooltip = Tooltip::new("Helpful hint", visible);
    let mut tt_state = tooltip.build();
    let mut arena = LayoutArena::new();
    let root = tooltip.layout(&mut tt_state, &mut arena);
    LayoutEngine::compute_layout(&mut arena, root, Size::new(300.0, 100.0));

    let mut comp = ClipRecordingCompositor::default();
    {
        let mut ctx = PaintContext::new(&mut comp, &arena, &theme, &dirty);
        tooltip.paint(&tt_state, &mut ctx);
    }
    assert!(
        comp.filled_rounded_rects
            .iter()
            .any(|(_, _, col)| *col == theme.colors.popover)
    );
    assert!(
        comp.drawn_texts
            .iter()
            .any(|(t, _, _, col)| t == "Helpful hint" && *col == theme.colors.popover_foreground)
    );
    assert_eq!(comp.pushed_clips.len(), 1);
    assert_eq!(comp.popped_clips, 1);
}

#[test]
fn test_switch_symmetric_thumb_offset_and_slider_inset_with_border() {
    // 1. Switch symmetric thumb offset: 2.0 (unchecked) vs 22.0 (checked)
    let checked = Signal::new(false);
    let sw = Switch::new(checked);
    let mut sw_state = sw.build();
    assert!((sw_state.thumb_offset_x - 2.0).abs() < f32::EPSILON);

    checked.set(true);
    sw.rebuild(&mut sw_state);
    assert!((sw_state.thumb_offset_x - 22.0).abs() < f32::EPSILON);

    let sw_checked_state = Switch::new(Signal::new(true)).build();
    assert!((sw_checked_state.thumb_offset_x - 22.0).abs() < f32::EPSILON);

    // 2. Slider track inset by thumb radius and thumb border drawn
    let theme = Theme::dark();
    let dirty: [Rect; 0] = [];
    for val in [0.0_f32, 50.0, 100.0] {
        let sig = Signal::new(val);
        let slider = Slider::new(sig, 0.0, 100.0).width(Dimension::Points(200.0));
        let mut st = slider.build();
        let mut arena = LayoutArena::new();
        let root = slider.layout(&mut st, &mut arena);
        LayoutEngine::compute_layout(&mut arena, root, Size::new(300.0, 60.0));
        let bounds = arena.computed_bounds[st.node as usize];

        let mut comp = ClipRecordingCompositor::default();
        {
            let mut ctx = PaintContext::new(&mut comp, &arena, &theme, &dirty);
            slider.paint(&st, &mut ctx);
        }

        // Every painted rect must stay strictly within horizontal bounds [bounds.origin.x, bounds.origin.x + bounds.size.width]
        for (rect, _, _) in &comp.filled_rounded_rects {
            assert!(
                rect.origin.x >= bounds.origin.x - 0.01,
                "Slider rect overflowed left for val={val}: {rect:?} vs {bounds:?}"
            );
            assert!(
                rect.origin.x + rect.size.width <= bounds.origin.x + bounds.size.width + 0.01,
                "Slider rect overflowed right for val={val}: {rect:?} vs {bounds:?}"
            );
        }

        // Thumb has both outer border (16x16, primary) and inner fill (13x13, background)
        assert!(
            comp.filled_rounded_rects.iter().any(|(r, _, c)| {
                (r.size.width - 16.0).abs() < 0.01 && *c == theme.colors.primary
            })
        );
        assert!(comp.filled_rounded_rects.iter().any(|(r, _, c)| {
            (r.size.width - 13.0).abs() < 0.01 && *c == theme.colors.background
        }));
    }
}

#[test]
fn test_select_width_builder_clipping_and_hover_blending() {
    let selected = Signal::new("a".to_string());
    let is_open = Signal::new(true);
    let select = Select::new(
        selected,
        is_open,
        vec![
            SelectOption::new("a".to_string(), "A very long option label that clips"),
            SelectOption::new("b".to_string(), "Another option"),
        ],
        "Choose",
    )
    .width(Dimension::Points(240.0));

    let mut state = select.build();
    let mut arena = LayoutArena::new();
    let root = select.layout(&mut state, &mut arena);
    LayoutEngine::compute_layout(&mut arena, root, Size::new(400.0, 300.0));

    let bounds = arena.computed_bounds[state.node as usize];
    assert!((bounds.size.width - 240.0).abs() < 0.01);

    let theme = Theme::dark();
    let dirty: [Rect; 0] = [];
    let mut comp = ClipRecordingCompositor::default();
    {
        let mut ctx = PaintContext::new(&mut comp, &arena, &theme, &dirty).with_interaction(
            None,
            Some(state.node),
            None,
        );
        select.paint(&state, &mut ctx);
    }

    // Trigger base fill (input_bg) + opaque hover fill (accent)
    assert!(
        comp.filled_rounded_rects
            .iter()
            .any(|(_, _, c)| *c == theme.colors.input_bg)
    );
    assert!(
        comp.filled_rounded_rects
            .iter()
            .any(|(_, _, c)| *c == theme.colors.accent)
    );
    // 1 trigger label clip (width - 22.0) + 2 option label clips = 3 balanced clips
    assert_eq!(comp.pushed_clips.len(), 3);
    assert_eq!(comp.popped_clips, 3);
    assert!((comp.pushed_clips[0].0.size.width - (240.0 - 22.0)).abs() < 0.01);
}

#[test]
fn test_card_footer_padding_and_clipping_and_sidebar_item_width_and_clipping() {
    // 1. Card reserves bottom padding when footer.is_some() and clips interior
    let card_no_footer = Card::new("Summary").content(Text::new("Body"));
    let mut st_no = card_no_footer.build();
    let mut ar_no = LayoutArena::new();
    let root_no = card_no_footer.layout(&mut st_no, &mut ar_no);

    let card_with_footer = Card::new("Summary")
        .footer("Updated just now")
        .content(Text::new("Body"));
    let mut st_yes = card_with_footer.build();
    let mut ar_yes = LayoutArena::new();
    let root_yes = card_with_footer.layout(&mut st_yes, &mut ar_yes);
    LayoutEngine::compute_layout(&mut ar_yes, root_yes, Size::new(320.0, 200.0));

    assert!(
        ar_yes.styles[root_yes as usize].padding.bottom
            > ar_no.styles[root_no as usize].padding.bottom
    );

    let theme = Theme::dark();
    let dirty: [Rect; 0] = [];
    let mut comp = ClipRecordingCompositor::default();
    {
        let mut ctx = PaintContext::new(&mut comp, &ar_yes, &theme, &dirty);
        card_with_footer.paint(&st_yes, &mut ctx);
    }
    assert_eq!(comp.pushed_clips.len(), 1);
    assert_eq!(comp.popped_clips, 1);

    // 2. Sidebar item width is Dimension::Percent(1.0) and item labels are clipped
    let collapsed = Signal::new(false);
    let sidebar = Sidebar::new(
        collapsed,
        vec![
            SidebarItem::new("home", "Dashboard").mark_active(),
            SidebarItem::new("settings", "Workspace Settings"),
        ],
    );
    let mut sb_state = sidebar.build();
    let mut sb_arena = LayoutArena::new();
    let sb_root = sidebar.layout(&mut sb_state, &mut sb_arena);
    LayoutEngine::compute_layout(&mut sb_arena, sb_root, Size::new(240.0, 600.0));

    for &item_node in &sb_state.item_nodes {
        assert_eq!(
            sb_arena.styles[item_node as usize].width,
            Dimension::Percent(1.0)
        );
        assert!(sb_arena.computed_bounds[item_node as usize].size.width > 200.0);
    }

    let mut sb_comp = ClipRecordingCompositor::default();
    {
        let mut ctx = PaintContext::new(&mut sb_comp, &sb_arena, &theme, &dirty);
        sidebar.paint(&sb_state, &mut ctx);
    }
    assert_eq!(sb_comp.pushed_clips.len(), 2);
    assert_eq!(sb_comp.popped_clips, 2);
}

#[test]
fn test_tabs_explicit_measured_width_centering_and_clipping() {
    let active = Signal::new(0usize);
    let tabs = Tabs::new(
        vec![
            TabItem::new("a", "OK"),
            TabItem::new("b", "Account Settings"),
        ],
        active,
        |_| Text::new("Tab Content"),
    );

    let mut state = tabs.build();
    let mut arena = LayoutArena::new();
    let root = tabs.layout(&mut state, &mut arena);
    LayoutEngine::compute_layout(&mut arena, root, Size::new(500.0, 200.0));

    // "OK" -> (2 * 8 + 28).max(64) = 64.0
    // "Account Settings" (16 chars) -> (16 * 8 + 28).max(64) = 156.0
    let w0 = arena.computed_bounds[state.tab_nodes[0] as usize]
        .size
        .width;
    let w1 = arena.computed_bounds[state.tab_nodes[1] as usize]
        .size
        .width;
    assert!((w0 - 64.0).abs() < 0.01);
    assert!((w1 - 156.0).abs() < 0.01);

    let theme = Theme::dark();
    let dirty: [Rect; 0] = [];
    let mut comp = ClipRecordingCompositor::default();
    {
        let mut ctx = PaintContext::new(&mut comp, &arena, &theme, &dirty);
        tabs.paint(&state, &mut ctx);
    }
    assert_eq!(comp.pushed_clips.len(), 2);
    assert_eq!(comp.popped_clips, 2);
}

#[test]
fn test_sheet_and_modal_scrim_and_sheet_close_button_layout() {
    let theme = Theme::dark();
    let dirty: [Rect; 0] = [];

    // 1. Sheet: close button on right edge (no overlap with title) + scrim backdrop
    let sheet_open = Signal::new(true);
    let sheet = Sheet::new(sheet_open, "Inspector Panel")
        .side(SheetSide::Right)
        .size(360.0);
    let mut sh_state = sheet.build();
    let mut arena = LayoutArena::new();
    let root = sheet.layout(&mut sh_state, &mut arena);
    LayoutEngine::compute_layout(&mut arena, root, Size::new(1000.0, 700.0));

    let panel_bounds = arena.computed_bounds[sh_state.panel_node as usize];
    let close_bounds = arena.computed_bounds[sh_state.close_node as usize];
    // Close button must be on the right side of the panel, well clear of the title at origin.x + 16
    assert!(
        close_bounds.origin.x >= panel_bounds.origin.x + panel_bounds.size.width - 50.0,
        "Close button should be right-aligned: close={close_bounds:?}, panel={panel_bounds:?}"
    );

    let mut comp = ClipRecordingCompositor::default();
    {
        let mut ctx = PaintContext::new(&mut comp, &arena, &theme, &dirty);
        sheet.paint(&sh_state, &mut ctx);
    }
    assert!(
        comp.filled_rects
            .iter()
            .any(|(_, c)| *c == theme.colors.scrim)
    );
    assert_eq!(comp.pushed_clips.len(), comp.popped_clips);

    // 2. Modal uses theme.colors.scrim for backdrop
    let modal_open = Signal::new(true);
    let modal = Modal::new(modal_open, Text::new("Confirm action"));
    let mut m_state = modal.build();
    let mut m_arena = LayoutArena::new();
    let m_root = modal.layout(&mut m_state, &mut m_arena);
    LayoutEngine::compute_layout(&mut m_arena, m_root, Size::new(800.0, 600.0));

    let mut m_comp = ClipRecordingCompositor::default();
    {
        let mut ctx = PaintContext::new(&mut m_comp, &m_arena, &theme, &dirty);
        modal.paint(&m_state, &mut ctx);
    }
    assert!(
        m_comp
            .filled_rects
            .iter()
            .any(|(_, c)| *c == theme.colors.scrim)
    );
}

#[test]
fn test_datagrid_cell_clipping_and_rich_text_editor_focus_caret_and_clipping() {
    let theme = Theme::light();
    let dirty: [Rect; 0] = [];

    // 1. DataGrid clips every header cell and body cell
    let columns = vec![
        GridColumn::new("c1", "Column One").width(120.0),
        GridColumn::new("c2", "Column Two").width(180.0),
    ];
    let grid = DataGrid::new(
        columns,
        DataGridDimensions {
            total_rows: 2,
            row_height: 30.0,
            viewport_width: 300.0,
            viewport_height: 150.0,
        },
        Signal::new(0.0),
        Signal::new(0.0),
        |coords| Text::new(format!("R{}C{}", coords.row, coords.col)),
    );
    let mut g_state = grid.build();
    let mut g_arena = LayoutArena::new();
    let g_root = grid.layout(&mut g_state, &mut g_arena);
    LayoutEngine::compute_layout(&mut g_arena, g_root, Size::new(300.0, 150.0));

    let mut g_comp = ClipRecordingCompositor::default();
    {
        let mut ctx = PaintContext::new(&mut g_comp, &g_arena, &theme, &dirty);
        grid.paint(&g_state, &mut ctx);
    }
    // 2 header cells + (2 rows * 2 cols = 4 body cells) = 6 clip push/pop pairs
    assert_eq!(g_comp.pushed_clips.len(), 6);
    assert_eq!(g_comp.popped_clips, 6);
    assert!((g_comp.pushed_clips[0].0.size.width - 120.0).abs() < 0.01);
    assert!((g_comp.pushed_clips[1].0.size.width - 180.0).abs() < 0.01);

    // 2. RichTextEditor: themed selection, no caret when unfocused, caret when focused, clipped lines
    let mut editor = RichTextEditor::with_text("line one\nline two");
    editor.select_all();
    let mut e_state = editor.build();
    let mut e_arena = LayoutArena::new();
    let e_root = editor.layout(&mut e_state, &mut e_arena);
    LayoutEngine::compute_layout(&mut e_arena, e_root, Size::new(400.0, 200.0));

    // Unfocused: selection highlight uses theme.colors.primary.with_alpha(96), no 2px caret rect
    let mut comp_unfocused = ClipRecordingCompositor::default();
    {
        let mut ctx = PaintContext::new(&mut comp_unfocused, &e_arena, &theme, &dirty);
        editor.paint(&e_state, &mut ctx);
    }
    assert!(
        comp_unfocused
            .filled_rects
            .iter()
            .any(|(_, c)| *c == theme.colors.primary.with_alpha(96))
    );
    assert!(
        !comp_unfocused
            .filled_rects
            .iter()
            .any(|(r, _)| (r.size.width - 2.0).abs() < f32::EPSILON)
    );
    assert_eq!(comp_unfocused.pushed_clips.len(), 1);
    assert_eq!(comp_unfocused.popped_clips, 1);

    // Focused: paints both left focus border and 2px caret
    let mut comp_focused = ClipRecordingCompositor::default();
    {
        let mut ctx = PaintContext::new(&mut comp_focused, &e_arena, &theme, &dirty)
            .with_interaction(Some(e_state.node), None, None);
        editor.paint(&e_state, &mut ctx);
    }
    let caret_count = comp_focused
        .filled_rects
        .iter()
        .filter(|(r, c)| (r.size.width - 2.0).abs() < f32::EPSILON && *c == theme.colors.primary)
        .count();
    assert_eq!(caret_count, 2); // left focus accent bar + cursor caret
    assert_eq!(comp_focused.pushed_clips.len(), 1);
    assert_eq!(comp_focused.popped_clips, 1);
}
