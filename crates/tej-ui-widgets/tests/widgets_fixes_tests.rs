//! Comprehensive unit tests for `tej-ui-widgets` correctness and rendering fixes.

use tej_ui_core::Signal;
use tej_ui_hal::{Color, SurfaceCompositor, UiEvent};
use tej_ui_layout::{LayoutArena, LayoutEngine, Point, Rect, Size};
use tej_ui_view::{EventDispatcher, PaintContext, Text, Theme, View};
use tej_ui_widgets::{
    CursorPos, DataGrid, DataGridDimensions, EditAction, GridColumn, InputMask, Modal, PinState,
    RichTextEditor, TextBuffer, TreeNode, UndoStack, VirtualizedTreeIndex, max_length, min_length,
};

#[derive(Debug, Default)]
struct RecordingCompositor {
    filled_rects: Vec<(Rect, Color)>,
    drawn_texts: Vec<(String, Point)>,
    drawn_lines: Vec<(Point, Point)>,
}

impl SurfaceCompositor for RecordingCompositor {
    fn begin_frame(&mut self, _dirty_rects: &[Rect]) {}

    fn fill_rect(&mut self, rect: Rect, color: Color) {
        self.filled_rects.push((rect, color));
    }

    fn fill_rounded_rect(&mut self, rect: Rect, _radius: f32, color: Color) {
        self.filled_rects.push((rect, color));
    }

    fn draw_line(&mut self, p1: Point, p2: Point, _stroke_width: f32, _color: Color) {
        self.drawn_lines.push((p1, p2));
    }

    fn draw_text(&mut self, text: &str, origin: Point, _font_size: f32, _color: Color) {
        self.drawn_texts.push((text.to_string(), origin));
    }

    fn end_frame(&mut self) {}
}

#[test]
fn test_input_mask_apply_keystroke_multibyte_utf8_no_panic() {
    let mask = InputMask::parse("AAA-AAA");
    // Multi-byte UTF-8 letters ('é' = 2 bytes, 'ü' = 2 bytes)
    let (formatted, cursor) = mask.apply_keystroke("éü", 1, 'ö');
    assert_eq!(formatted, "éöü");
    assert_eq!(cursor, 2);

    // Insert at end after multi-byte characters
    let (formatted2, _) = mask.apply_keystroke("éöü", 3, 'ä');
    assert_eq!(formatted2, "éöü-ä");
}

#[test]
fn test_validators_min_and_max_length_use_char_count() {
    let min_v = min_length(3, "too short");
    let max_v = max_length(3, "too long");

    // "éé" is 4 UTF-8 bytes but only 2 characters -> fails min_length(3)
    assert!(min_v(&"éé".to_string()).is_some());
    // "ééé" is 6 UTF-8 bytes and 3 characters -> passes both min_length(3) and max_length(3)
    assert!(min_v(&"ééé".to_string()).is_none());
    assert!(max_v(&"ééé".to_string()).is_none());
    // "🎉🎉🎉🎉" is 4 characters -> fails max_length(3)
    assert!(max_v(&"🎉🎉🎉🎉".to_string()).is_some());
}

#[test]
fn test_text_buffer_preserves_trailing_newlines() {
    let buf = TextBuffer::from("hello\n");
    assert_eq!(buf.line_count(), 2);
    assert_eq!(buf.lines(), &["hello".to_string(), String::new()]);
    assert_eq!(buf.text(), "hello\n");

    let only_nl = TextBuffer::from("\n");
    assert_eq!(only_nl.line_count(), 2);
    assert_eq!(only_nl.text(), "\n");

    let crlf = TextBuffer::from("line1\r\nline2\r\n");
    assert_eq!(crlf.line_count(), 3);
    assert_eq!(crlf.text(), "line1\nline2\n");
}

#[test]
fn test_undo_stack_zero_capacity_does_not_panic() {
    let mut stack = UndoStack::new(0);
    stack.push(EditAction::Insert {
        pos: CursorPos::zero(),
        text: "abc".to_string(),
    });
    assert!(!stack.can_undo());

    stack.begin_transaction();
    stack.push(EditAction::Insert {
        pos: CursorPos::zero(),
        text: "x".to_string(),
    });
    stack.commit_transaction();
    assert!(!stack.can_undo());
}

#[test]
fn test_rich_text_editor_undo_backspace_delete_and_atomic_replace() {
    let mut editor = RichTextEditor::with_text("hello");
    editor.move_cursor_right(false);
    editor.move_cursor_right(false);
    // Cursor at col 2 ('he|llo')

    // Collapsed backspace deletes 'e' and records undo history
    editor.backspace();
    assert_eq!(editor.text(), "hllo");
    assert!(editor.undo());
    assert_eq!(editor.text(), "hello");

    // Collapsed delete_forward deletes 'l' and records undo history
    editor.delete_forward();
    assert_eq!(editor.text(), "helo");
    assert!(editor.undo());
    assert_eq!(editor.text(), "hello");

    // Replacing a selection via insert_text is a single atomic undo step
    editor.select_all();
    editor.insert_text("world");
    assert_eq!(editor.text(), "world");
    assert!(editor.undo());
    assert_eq!(editor.text(), "hello");
    assert!(editor.redo());
    assert_eq!(editor.text(), "world");
}

#[test]
fn test_rich_text_editor_paint_renders_caret_and_selection_highlight() {
    let mut editor = RichTextEditor::with_text("hello world");
    editor.select_all();

    let mut state = editor.build();
    let mut arena = LayoutArena::new();
    let root = editor.layout(&mut state, &mut arena);
    LayoutEngine::compute_layout(&mut arena, root, Size::new(400.0, 200.0));

    let mut compositor = RecordingCompositor::default();
    let theme = Theme::dark();
    let dirty: [Rect; 0] = [];
    {
        let mut ctx = PaintContext::new(&mut compositor, &arena, &theme, &dirty);
        editor.paint(&state, &mut ctx);
    }

    // Must draw background, selection highlight rect, and 2px-wide cursor caret rect
    assert!(
        compositor
            .filled_rects
            .iter()
            .any(|(r, _)| (r.size.width - 2.0).abs() < f32::EPSILON)
    );
    assert!(
        compositor
            .filled_rects
            .iter()
            .any(|(_, c)| *c == Color::rgba(59, 130, 246, 96))
    );
}

#[test]
fn test_datagrid_guards_pinned_columns_and_renders() {
    let columns = vec![
        GridColumn::new("id", "ID").width(150.0).pin(PinState::Left),
        GridColumn::new("col_a", "Col A").width(150.0),
        GridColumn::new("col_b", "Col B").width(150.0),
        GridColumn::new("col_c", "Col C").width(150.0),
    ];

    // 1. row_height <= 0.0 returns (0, 0) safely without NaN/panic
    let zero_grid = DataGrid::new(
        columns.clone(),
        DataGridDimensions {
            total_rows: 100,
            row_height: 0.0,
            viewport_width: 300.0,
            viewport_height: 300.0,
        },
        Signal::new(0.0),
        Signal::new(0.0),
        |coords| Text::new(format!("{},{}", coords.row, coords.col)),
    );
    assert_eq!(zero_grid.visible_rows(), (0, 0));

    // 2. Left-pinned column (150px) reduces center viewport from 300px to 150px,
    // so only col_a (0..150) is visible at scroll_x = 0, not col_b (150..300) or col_c (300..450)
    let grid = DataGrid::new(
        columns,
        DataGridDimensions {
            total_rows: 10,
            row_height: 30.0,
            viewport_width: 300.0,
            viewport_height: 200.0,
        },
        Signal::new(0.0),
        Signal::new(0.0),
        |coords| Text::new(format!("R{}C{}", coords.row, coords.col)),
    );
    let visible_cols = grid.visible_columns();
    assert_eq!(visible_cols, vec![0, 1]);

    // 3. layout() and paint() render column headers and visible rows
    let mut state = grid.build();
    let mut arena = LayoutArena::new();
    let root = grid.layout(&mut state, &mut arena);
    LayoutEngine::compute_layout(&mut arena, root, Size::new(300.0, 200.0));

    let mut compositor = RecordingCompositor::default();
    let theme = Theme::dark();
    let dirty: [Rect; 0] = [];
    {
        let mut ctx = PaintContext::new(&mut compositor, &arena, &theme, &dirty);
        grid.paint(&state, &mut ctx);
    }

    assert!(compositor.drawn_texts.iter().any(|(t, _)| t == "ID"));
    assert!(compositor.drawn_texts.iter().any(|(t, _)| t == "Col A"));
    assert!(!compositor.filled_rects.is_empty());
}

#[test]
fn test_tree_virtualizer_clamps_first_row_no_inverted_range() {
    let roots = vec![
        TreeNode::leaf("1", "Root 1"),
        TreeNode::leaf("2", "Root 2"),
        TreeNode::leaf("3", "Root 3"),
    ];
    let index = VirtualizedTreeIndex::from_roots(&roots);

    // Extreme scroll_y far beyond total content height
    let range = index.compute_visible_range(100_000.0, 24.0, 400.0, 2);
    assert!(range.start <= range.end);
    assert_eq!(range, 3..3);

    // Zero/negative row_height returns empty range safely
    assert_eq!(index.compute_visible_range(100.0, 0.0, 400.0, 2), 0..0);
    assert_eq!(index.compute_visible_range(100.0, -10.0, 400.0, 2), 0..0);
}

#[test]
fn test_modal_escape_key_dismissal() {
    let is_open = Signal::new(true);
    let modal = Modal::new(is_open, Text::new("Modal Content"));

    let mut state = modal.build();
    let mut arena = LayoutArena::new();
    let root = modal.layout(&mut state, &mut arena);
    LayoutEngine::compute_layout(&mut arena, root, Size::new(800.0, 600.0));

    let mut dispatcher = EventDispatcher::new();
    modal.register_events(&state, &mut dispatcher);
    dispatcher.focus(Some(state.node));

    // Non-Escape key should not dismiss modal
    dispatcher.dispatch(&UiEvent::KeyDown { key_code: 65 }, &arena, root);
    assert!(is_open.get());

    // Escape key (27) should dismiss modal
    dispatcher.dispatch(&UiEvent::KeyDown { key_code: 27 }, &arena, root);
    assert!(!is_open.get());
}
