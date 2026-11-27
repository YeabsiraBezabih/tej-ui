use tej_ui_hal::{Color, SurfaceCompositor};
use tej_ui_icons::{
    ActionIcon, DeviceIcon, Icon, LucideIcon, NavigationIcon, PathCommand, SystemIcon, VectorPath,
};
use tej_ui_layout::{Point, Rect, Size};
use tej_ui_view::View;

#[test]
fn test_svg_path_parsing() {
    let d = "M10 20 L30 40 Z";
    let path = VectorPath::parse_svg(d);
    assert_eq!(path.commands.len(), 3);
    assert_eq!(
        path.commands[0],
        PathCommand::MoveTo(Point::new(10.0, 20.0))
    );
    assert_eq!(
        path.commands[1],
        PathCommand::LineTo(Point::new(30.0, 40.0))
    );
    assert_eq!(path.commands[2], PathCommand::Close);

    let bounds = path.bounds();
    assert_eq!(bounds.origin.x, 10.0);
    assert_eq!(bounds.origin.y, 20.0);
    assert_eq!(bounds.size.width, 20.0);
    assert_eq!(bounds.size.height, 20.0);
}

#[test]
fn test_svg_parser_arcs_curves_and_implicit_repeats() {
    // Implicit repeated pairs after M (treated as L) and m (treated as l)
    let implicit_m = VectorPath::parse_svg("M 1 2 3 4 5 6 m 1 1 2 3");
    assert_eq!(implicit_m.commands.len(), 5);
    assert_eq!(
        implicit_m.commands[0],
        PathCommand::MoveTo(Point::new(1.0, 2.0))
    );
    assert_eq!(
        implicit_m.commands[1],
        PathCommand::LineTo(Point::new(3.0, 4.0))
    );
    assert_eq!(
        implicit_m.commands[2],
        PathCommand::LineTo(Point::new(5.0, 6.0))
    );
    assert_eq!(
        implicit_m.commands[3],
        PathCommand::MoveTo(Point::new(6.0, 7.0))
    );
    assert_eq!(
        implicit_m.commands[4],
        PathCommand::LineTo(Point::new(8.0, 10.0))
    );

    // Implicit repeated pairs after L/l
    let implicit_l = VectorPath::parse_svg("M0 0 L10 0 10 10 l-5 0 0 -5");
    assert_eq!(implicit_l.commands.len(), 5);
    assert_eq!(
        implicit_l.commands[4],
        PathCommand::LineTo(Point::new(5.0, 5.0))
    );

    // Cubic C/c and smooth cubic S/s
    let cubic = VectorPath::parse_svg("M0 0 C1 2 3 4 5 6 S9 8 10 10 c1 0 2 1 3 1 s2 1 3 0");
    assert_eq!(cubic.commands.len(), 5);
    assert!(matches!(
        cubic.commands[2],
        PathCommand::CubicTo { ctrl1, ctrl2, end }
            if (ctrl1.x - 7.0).abs() < 1e-4
                && (ctrl1.y - 8.0).abs() < 1e-4
                && (ctrl2.x - 9.0).abs() < 1e-4
                && (end.x - 10.0).abs() < 1e-4
    ));

    // Quadratic Q/q and smooth quad T/t
    let quad = VectorPath::parse_svg("M0 0 Q5 10 10 0 T20 0 q2 -5 4 0 t4 0");
    assert_eq!(quad.commands.len(), 5);
    assert!(matches!(
        quad.commands[2],
        PathCommand::QuadTo { control, end }
            if (control.x - 15.0).abs() < 1e-4
                && (control.y - -10.0).abs() < 1e-4
                && (end.x - 20.0).abs() < 1e-4
    ));

    // Elliptical arc A/a including compact flags without separators (01)
    let arc = VectorPath::parse_svg("M2 12 A10 10 0 0 1 22 12 a10 10 0 01-20 0");
    let cubic_count = arc
        .commands
        .iter()
        .filter(|c| matches!(c, PathCommand::CubicTo { .. }))
        .count();
    assert!(
        cubic_count >= 4,
        "full circle from two 180-deg arcs should emit >= 4 cubic segments, got {cubic_count}"
    );
}

#[test]
fn test_all_128_lucide_icons_parse_valid_geometry() {
    let all_icons: [LucideIcon; 128] = [
        // Navigation (23)
        LucideIcon::ChevronDown,
        LucideIcon::ChevronUp,
        LucideIcon::ChevronLeft,
        LucideIcon::ChevronRight,
        LucideIcon::ChevronsDown,
        LucideIcon::ChevronsUp,
        LucideIcon::ChevronsLeft,
        LucideIcon::ChevronsRight,
        LucideIcon::ArrowDown,
        LucideIcon::ArrowUp,
        LucideIcon::ArrowLeft,
        LucideIcon::ArrowRight,
        LucideIcon::ArrowUpRight,
        LucideIcon::ArrowDownRight,
        LucideIcon::CornerUpLeft,
        LucideIcon::CornerDownRight,
        LucideIcon::Home,
        LucideIcon::Menu,
        LucideIcon::MoreHorizontal,
        LucideIcon::MoreVertical,
        LucideIcon::ExternalLink,
        LucideIcon::Compass,
        LucideIcon::MapPin,
        // Actions (20)
        LucideIcon::Plus,
        LucideIcon::Minus,
        LucideIcon::Check,
        LucideIcon::X,
        LucideIcon::Trash,
        LucideIcon::Trash2,
        LucideIcon::Edit,
        LucideIcon::Edit2,
        LucideIcon::Copy,
        LucideIcon::Save,
        LucideIcon::Share,
        LucideIcon::Download,
        LucideIcon::Upload,
        LucideIcon::RefreshCw,
        LucideIcon::RefreshCcw,
        LucideIcon::Filter,
        LucideIcon::ZoomIn,
        LucideIcon::ZoomOut,
        LucideIcon::Maximize,
        LucideIcon::Minimize,
        // Media (16)
        LucideIcon::Play,
        LucideIcon::Pause,
        LucideIcon::Square,
        LucideIcon::SkipForward,
        LucideIcon::SkipBack,
        LucideIcon::Volume,
        LucideIcon::Volume1,
        LucideIcon::Volume2,
        LucideIcon::VolumeX,
        LucideIcon::Mic,
        LucideIcon::MicOff,
        LucideIcon::Video,
        LucideIcon::VideoOff,
        LucideIcon::Camera,
        LucideIcon::Image,
        LucideIcon::Music,
        // Devices & Hardware (15)
        LucideIcon::Laptop,
        LucideIcon::Monitor,
        LucideIcon::Smartphone,
        LucideIcon::Tablet,
        LucideIcon::Tv,
        LucideIcon::HardDrive,
        LucideIcon::Cpu,
        LucideIcon::Battery,
        LucideIcon::BatteryCharging,
        LucideIcon::Wifi,
        LucideIcon::WifiOff,
        LucideIcon::Bluetooth,
        LucideIcon::Server,
        LucideIcon::Database,
        LucideIcon::Terminal,
        // Communication (12)
        LucideIcon::Mail,
        LucideIcon::Inbox,
        LucideIcon::Send,
        LucideIcon::AtSign,
        LucideIcon::MessageSquare,
        LucideIcon::MessageCircle,
        LucideIcon::Phone,
        LucideIcon::PhoneCall,
        LucideIcon::PhoneOff,
        LucideIcon::Bell,
        LucideIcon::BellOff,
        LucideIcon::Globe,
        // Editor & Text (13)
        LucideIcon::Bold,
        LucideIcon::Italic,
        LucideIcon::Underline,
        LucideIcon::AlignLeft,
        LucideIcon::AlignCenter,
        LucideIcon::AlignRight,
        LucideIcon::AlignJustify,
        LucideIcon::List,
        LucideIcon::Code,
        LucideIcon::Link,
        LucideIcon::Link2,
        LucideIcon::Scissors,
        LucideIcon::Hash,
        // System & Status (29)
        LucideIcon::Search,
        LucideIcon::Settings,
        LucideIcon::Tool,
        LucideIcon::User,
        LucideIcon::Users,
        LucideIcon::UserPlus,
        LucideIcon::UserCheck,
        LucideIcon::UserX,
        LucideIcon::Lock,
        LucideIcon::Unlock,
        LucideIcon::Key,
        LucideIcon::Shield,
        LucideIcon::ShieldCheck,
        LucideIcon::Eye,
        LucideIcon::EyeOff,
        LucideIcon::Sun,
        LucideIcon::Moon,
        LucideIcon::Info,
        LucideIcon::HelpCircle,
        LucideIcon::AlertCircle,
        LucideIcon::AlertTriangle,
        LucideIcon::CheckCircle,
        LucideIcon::XCircle,
        LucideIcon::Clock,
        LucideIcon::Calendar,
        LucideIcon::Bookmark,
        LucideIcon::Star,
        LucideIcon::Heart,
        LucideIcon::Flag,
    ];

    assert_eq!(all_icons.len(), 128);

    for icon in all_icons {
        let d = icon.svg_path();
        assert!(!d.is_empty(), "{icon:?} has empty SVG path");

        // Verify no unknown command letters exist in the catalog SVG data
        for ch in d.chars() {
            if ch.is_ascii_alphabetic() {
                assert!(
                    matches!(
                        ch,
                        'M' | 'm'
                            | 'L'
                            | 'l'
                            | 'H'
                            | 'h'
                            | 'V'
                            | 'v'
                            | 'C'
                            | 'c'
                            | 'S'
                            | 's'
                            | 'Q'
                            | 'q'
                            | 'T'
                            | 't'
                            | 'A'
                            | 'a'
                            | 'Z'
                            | 'z'
                            | 'e'
                            | 'E'
                    ),
                    "{icon:?} contains unsupported SVG token '{ch}'"
                );
            }
        }

        let path = icon.to_vector_path();
        assert!(
            !path.commands.is_empty(),
            "{icon:?} parsed into empty VectorPath"
        );

        let has_drawable = path.commands.iter().any(|cmd| {
            matches!(
                cmd,
                PathCommand::LineTo(_) | PathCommand::QuadTo { .. } | PathCommand::CubicTo { .. }
            )
        });
        assert!(
            has_drawable,
            "{icon:?} has no drawable segments (only MoveTo/Close)"
        );

        let has_arc_or_curve_token = d
            .chars()
            .any(|c| matches!(c, 'A' | 'a' | 'C' | 'c' | 'S' | 's' | 'Q' | 'q' | 'T' | 't'));
        if has_arc_or_curve_token {
            let has_curve_cmd = path.commands.iter().any(|cmd| {
                matches!(
                    cmd,
                    PathCommand::QuadTo { .. } | PathCommand::CubicTo { .. }
                )
            });
            assert!(
                has_curve_cmd,
                "{icon:?} has arc/curve tokens in SVG `{d}` but produced zero QuadTo/CubicTo commands"
            );
        }

        // Verify all coordinates are finite
        for cmd in &path.commands {
            match *cmd {
                PathCommand::MoveTo(p) | PathCommand::LineTo(p) => {
                    assert!(
                        p.x.is_finite() && p.y.is_finite(),
                        "{icon:?} non-finite point"
                    );
                }
                PathCommand::QuadTo { control, end } => {
                    assert!(
                        control.x.is_finite()
                            && control.y.is_finite()
                            && end.x.is_finite()
                            && end.y.is_finite(),
                        "{icon:?} non-finite quad"
                    );
                }
                PathCommand::CubicTo { ctrl1, ctrl2, end } => {
                    assert!(
                        ctrl1.x.is_finite()
                            && ctrl1.y.is_finite()
                            && ctrl2.x.is_finite()
                            && ctrl2.y.is_finite()
                            && end.x.is_finite()
                            && end.y.is_finite(),
                        "{icon:?} non-finite cubic"
                    );
                }
                PathCommand::Close => {}
            }
        }
    }
}

#[test]
fn test_expanded_lucide_catalog_coverage() {
    // Navigation icons
    assert!(!NavigationIcon::ChevronDown.svg_path().is_empty());
    assert!(!NavigationIcon::ArrowUpRight.svg_path().is_empty());
    assert!(!NavigationIcon::Compass.svg_path().is_empty());

    // Action icons
    assert!(!ActionIcon::Plus.svg_path().is_empty());
    assert!(!ActionIcon::Trash2.svg_path().is_empty());
    assert!(!ActionIcon::Share.svg_path().is_empty());

    // Device icons
    assert!(!DeviceIcon::Laptop.svg_path().is_empty());
    assert!(!DeviceIcon::Cpu.svg_path().is_empty());
    assert!(!DeviceIcon::Database.svg_path().is_empty());

    // System icons
    assert!(!SystemIcon::ShieldCheck.svg_path().is_empty());
    assert!(!SystemIcon::Star.svg_path().is_empty());
    assert!(!SystemIcon::AlertTriangle.svg_path().is_empty());

    // Unified LucideIcon conversions
    assert!(!LucideIcon::Search.to_vector_path().commands.is_empty());
    assert!(!LucideIcon::Laptop.to_vector_path().commands.is_empty());
    assert!(!LucideIcon::ShieldCheck.to_vector_path().commands.is_empty());
    assert!(!LucideIcon::Heart.to_vector_path().commands.is_empty());
}

#[derive(Debug, Default)]
struct MockCompositor {
    lines: Vec<(Point, Point)>,
}

impl SurfaceCompositor for MockCompositor {
    fn begin_frame(&mut self, _dirty_rects: &[Rect]) {}
    fn fill_rect(&mut self, _rect: Rect, _color: Color) {}
    fn fill_rounded_rect(&mut self, _rect: Rect, _radius: f32, _color: Color) {}
    fn draw_line(&mut self, p1: Point, p2: Point, _stroke_width: f32, _color: Color) {
        self.lines.push((p1, p2));
    }
    fn draw_text(&mut self, _text: &str, _origin: Point, _font_size: f32, _color: Color) {}
    fn end_frame(&mut self) {}
}

#[test]
fn test_icon_paint_draws_close_segment_and_flattens_curves() {
    // Triangle closed with Z: must draw 3 segments (including closing segment back to start)
    let tri = Icon::from_svg("M0 0 L24 0 L12 24 Z").size(24.0);
    let mut comp = MockCompositor::default();
    tri.paint(&mut comp, Rect::new(0.0, 0.0, 24.0, 24.0));
    assert_eq!(comp.lines.len(), 3);
    assert_eq!(comp.lines[2].0, Point::new(12.0, 24.0));
    assert_eq!(comp.lines[2].1, Point::new(0.0, 0.0));

    // Quadratic curve must be flattened into multiple segments (> 1)
    let quad = Icon::from_svg("M0 12 Q12 0 24 12").size(24.0);
    let mut quad_comp = MockCompositor::default();
    quad.paint(&mut quad_comp, Rect::new(0.0, 0.0, 24.0, 24.0));
    assert!(
        quad_comp.lines.len() >= 4,
        "QuadTo should flatten into >= 4 segments, got {}",
        quad_comp.lines.len()
    );

    // Cubic curve must be flattened into multiple segments (> 1)
    let cubic = Icon::from_svg("M0 12 C6 0 18 24 24 12").size(24.0);
    let mut cubic_comp = MockCompositor::default();
    cubic.paint(&mut cubic_comp, Rect::new(0.0, 0.0, 24.0, 24.0));
    assert!(
        cubic_comp.lines.len() >= 6,
        "CubicTo should flatten into >= 6 segments, got {}",
        cubic_comp.lines.len()
    );
}

#[test]
fn test_icon_view_widget_and_paint() {
    let icon = Icon::new(LucideIcon::ChevronDown)
        .size(16.0)
        .stroke_width(1.5)
        .color(Color::rgb(100, 100, 100));

    assert!(!icon.vector_path().commands.is_empty());

    let mut state = icon.build();
    assert_eq!(state.bounds.size, Size::new(16.0, 16.0));

    icon.rebuild(&mut state);
    assert_eq!(state.redraw_count, 1);

    let mut comp = MockCompositor::default();
    icon.paint(&mut comp, Rect::new(10.0, 10.0, 16.0, 16.0));
    assert!(!comp.lines.is_empty());
}

#[test]
fn test_custom_svg_icon() {
    let custom = Icon::from_svg("M0 0 L10 10").size(24.0);
    assert!(matches!(
        custom.vector_path().commands[1],
        PathCommand::LineTo(_)
    ));
}
