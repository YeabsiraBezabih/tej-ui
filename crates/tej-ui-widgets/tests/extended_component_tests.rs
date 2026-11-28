use std::time::Duration;
use tej_ui_hal::Color;
use tej_ui_widgets::{
    AgentPlanState, AspectRatioPreset, CalendarHeatmapState, ConfirmButtonState, ContextMeterState,
    DockNode, DockSplitOrientation, DockTab, FlameGraphState, FlameNode, GanttState, GanttTask,
    GitCommit, GitGraphState, GradientEditorState, HistogramSliderState, ImageCompareState,
    KanbanCard, KanbanColumn, KanbanState, KnobState, MetricCardState, MetricTrend, OtpInputState,
    PipelineStage, PipelineState, PipelineStatus, ReactionBarState, ReasoningDrawerState,
    ScrubFieldState, SignaturePadState, StrokePoint, ToolCallState, ToolExecutionStatus, TraceSpan,
    TraceWaterfallState, TreemapNode, TreemapState, UptimeBar,
};

#[test]
fn test_ai_reasoning_and_tool_call_lifecycle() {
    let mut reasoning = ReasoningDrawerState::new("Agent thinking process");
    assert!(!reasoning.is_expanded);
    reasoning.toggle();
    assert!(reasoning.is_expanded);

    reasoning.append_thought("Analyzing requirements...");
    assert!(reasoning.token_count > 0);
    reasoning.finish_thinking(Duration::from_millis(1250));
    assert_eq!(reasoning.elapsed, Duration::from_millis(1250));

    let mut tool = ToolCallState::new("read_file", "call_123", r#"{"path":"main.rs"}"#);
    assert_eq!(tool.status, ToolExecutionStatus::Pending);
    tool.set_success("fn main() {}", 42);
    assert_eq!(tool.status, ToolExecutionStatus::Success);
    assert_eq!(tool.execution_duration_ms, 42);
    assert_eq!(tool.output_result.as_deref(), Some("fn main() {}"));
}

#[test]
fn test_agent_plan_and_context_meter() {
    let mut plan = AgentPlanState::new("Refactor Architecture");
    plan.add_step("Step 1", "Decompose modules");
    plan.add_step("Step 2", "Write unit tests");
    assert_eq!(plan.steps.len(), 2);
    assert_eq!(plan.progress_ratio(), 0.0);

    plan.complete_current_step();
    assert_eq!(plan.progress_ratio(), 0.5);

    let meter = ContextMeterState::new(45_000, 100_000, 20_000);
    assert!((meter.usage_ratio() - 0.45).abs() < 1e-4);
    assert_eq!(meter.formatted_summary(), "45.0k / 100.0k tokens");
}

#[test]
fn test_flame_graph_and_trace_waterfall() {
    let nodes = vec![
        FlameNode {
            name: "root".to_string(),
            start_time_us: 0,
            duration_us: 10_000,
            depth: 0,
            color: Color::WHITE,
        },
        FlameNode {
            name: "child".to_string(),
            start_time_us: 2_000,
            duration_us: 4_000,
            depth: 1,
            color: Color::WHITE,
        },
    ];
    let mut flame = FlameGraphState::new(nodes, 10_000);
    assert_eq!(flame.zoom_end_us, 10_000);
    flame.zoom_to_node(1);
    assert_eq!(flame.zoom_start_us, 2_000);
    assert_eq!(flame.zoom_end_us, 6_000);

    let spans = vec![TraceSpan {
        id: "span_1".to_string(),
        parent_id: None,
        service_name: "gateway".to_string(),
        operation_name: "HTTP GET".to_string(),
        start_time_ms: 0.0,
        duration_ms: 120.0,
        phases: Vec::new(),
        has_error: false,
    }];
    let waterfall = TraceWaterfallState::new(spans, 120.0);
    assert_eq!(waterfall.spans.len(), 1);
}

#[test]
fn test_git_graph_and_pipeline() {
    let commits = vec![GitCommit {
        hash_short: "abc1234".to_string(),
        message: "Initial commit".to_string(),
        author: "Developer".to_string(),
        timestamp_str: "2026-11-23".to_string(),
        lane: 0,
        parent_indices: Vec::new(),
        branch_tags: vec!["main".to_string()],
    }];
    let git = GitGraphState::new(commits);
    assert_eq!(git.commits.len(), 1);

    let stages = vec![
        PipelineStage {
            name: "Lint".to_string(),
            status: PipelineStatus::Passed,
            duration_seconds: 5,
            steps_total: 2,
            steps_passed: 2,
        },
        PipelineStage {
            name: "Build".to_string(),
            status: PipelineStatus::Running,
            duration_seconds: 15,
            steps_total: 4,
            steps_passed: 1,
        },
    ];
    let pipeline = PipelineState::new(stages);
    assert_eq!(pipeline.stages[0].status, PipelineStatus::Passed);
}

#[test]
fn test_workspace_docking_and_kanban() {
    let tab_a = DockTab {
        id: "editor".to_string(),
        title: "Editor".to_string(),
        is_closable: false,
    };
    let tab_b = DockTab {
        id: "terminal".to_string(),
        title: "Terminal".to_string(),
        is_closable: true,
    };
    let node_a = DockNode::Tabs {
        tabs: vec![tab_a],
        active_tab_index: 0,
    };
    let node_b = DockNode::Tabs {
        tabs: vec![tab_b],
        active_tab_index: 0,
    };
    let dock = tej_ui_widgets::DockManagerState::new_split(
        DockSplitOrientation::Horizontal,
        0.7,
        node_a,
        node_b,
    );
    match &dock.root {
        DockNode::Split {
            ratio, orientation, ..
        } => {
            assert_eq!(*orientation, DockSplitOrientation::Horizontal);
            assert!((ratio - 0.7).abs() < 1e-4);
        }
        DockNode::Tabs { .. } => panic!("Expected split root"),
    }

    let mut kanban = KanbanState::new(vec![
        KanbanColumn {
            id: "todo".to_string(),
            title: "To Do".to_string(),
            cards: vec![KanbanCard {
                id: "c1".to_string(),
                title: "Card 1".to_string(),
                description: "Desc".to_string(),
                tag: "FE".to_string(),
                tag_color: Color::WHITE,
                assignee_name: None,
            }],
        },
        KanbanColumn {
            id: "done".to_string(),
            title: "Done".to_string(),
            cards: Vec::new(),
        },
    ]);
    kanban.move_card("c1", 1, 0);
    assert_eq!(kanban.columns[0].cards.len(), 0);
    assert_eq!(kanban.columns[1].cards.len(), 1);
}

#[test]
fn test_visualizations_and_extended_inputs() {
    let heatmap = CalendarHeatmapState::new(vec![0, 5, 10, 20]);
    assert_eq!(heatmap.max_count, 20);
    assert!((heatmap.intensity_for_day(2) - 0.5).abs() < 1e-4);

    let treemap = TreemapState::new(vec![
        TreemapNode {
            name: "A".to_string(),
            value: 100.0,
            color: Color::WHITE,
            rect: tej_ui_layout::Rect::ZERO,
        },
        TreemapNode {
            name: "B".to_string(),
            value: 50.0,
            color: Color::WHITE,
            rect: tej_ui_layout::Rect::ZERO,
        },
    ]);
    assert_eq!(treemap.nodes[0].name, "A");

    let mut otp = OtpInputState::new(4);
    otp.input_digit('1');
    otp.input_digit('2');
    otp.input_digit('3');
    otp.input_digit('4');
    assert!(otp.is_complete());
    assert_eq!(otp.code_string(), "1234");
    otp.backspace();
    assert!(!otp.is_complete());
    assert_eq!(otp.code_string(), "123");

    let mut scrub = ScrubFieldState::new(10.0, 0.0, 100.0, 1.0);
    scrub.is_dragging = true;
    scrub.start_value = 10.0;
    scrub.on_drag_delta(25.0); // +5 units
    assert!((scrub.value - 15.0).abs() < 1e-4);
}

#[test]
fn test_media_and_creative_controls() {
    let mut sig = SignaturePadState::default();
    sig.begin_stroke(
        StrokePoint {
            x: 0.0,
            y: 0.0,
            pressure: 1.0,
        },
        Color::BLACK,
        2.0,
    );
    sig.append_point(StrokePoint {
        x: 10.0,
        y: 10.0,
        pressure: 1.0,
    });
    sig.end_stroke();
    assert_eq!(sig.strokes.len(), 1);
    assert_eq!(sig.strokes[0].points.len(), 2);

    let mut compare = ImageCompareState::default();
    assert_eq!(compare.split_position, 0.5);
    compare.update_split(0.75);
    assert_eq!(compare.split_position, 0.75);

    let kpi = MetricCardState::new("Revenue", "$120,400", 14.5);
    assert_eq!(kpi.trend, MetricTrend::Positive);

    let mut knob = KnobState {
        is_dragging: true,
        start_value: 0.5,
        ..Default::default()
    };
    knob.on_drag_delta_y(-20.0); // upwards drag increases
    assert!(knob.value > 0.5);

    assert_eq!(AspectRatioPreset::Landscape16x9.label(), "16:9");

    let grad = GradientEditorState::new_two_stop(Color::BLACK, Color::WHITE);
    assert_eq!(grad.stops.len(), 2);

    let slider = HistogramSliderState::new(vec![10, 20, 30], 0.2, 0.8);
    assert_eq!(slider.bucket_counts.len(), 3);

    let mut confirm = ConfirmButtonState::new("Delete", Duration::from_millis(500));
    confirm.is_holding = true;
    let fired = confirm.update_hold(Duration::from_millis(600));
    assert!(fired);
    assert!(confirm.is_confirmed);

    let mut reactions = ReactionBarState::default();
    reactions.toggle_reaction("🚀");
    assert_eq!(reactions.reactions.len(), 1);
    assert_eq!(reactions.reactions[0].count, 1);

    let gantt = GanttState::new(
        vec![GanttTask {
            id: "task1".to_string(),
            title: "Task 1".to_string(),
            start_day: 0.0,
            duration_days: 5.0,
            progress: 0.5,
            dependency_ids: Vec::new(),
            color: Color::WHITE,
        }],
        10.0,
    );
    assert_eq!(gantt.tasks.len(), 1);

    let uptime = UptimeBar::default();
    let col = uptime.color_for_uptime(99.95);
    assert_eq!(col, Color::from_rgb_u32(0x0012_B76A));
}
