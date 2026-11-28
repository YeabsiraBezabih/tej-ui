//! Unified catalog of 120+ standard Lucide icons categorized across 6 domains.

pub mod actions;
pub mod communication;
pub mod devices;
pub mod editor;
pub mod media;
pub mod navigation;
pub mod system;

pub use actions::ActionIcon;
pub use communication::CommunicationIcon;
pub use devices::DeviceIcon;
pub use editor::EditorIcon;
pub use media::MediaIcon;
pub use navigation::NavigationIcon;
pub use system::SystemIcon;

use crate::path::VectorPath;

/// Flat unified enumeration of 120+ canonical Lucide icons.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LucideIcon {
    // Navigation
    ChevronDown,
    ChevronUp,
    ChevronLeft,
    ChevronRight,
    ChevronsDown,
    ChevronsUp,
    ChevronsLeft,
    ChevronsRight,
    ArrowDown,
    ArrowUp,
    ArrowLeft,
    ArrowRight,
    ArrowUpRight,
    ArrowDownRight,
    CornerUpLeft,
    CornerDownRight,
    Home,
    Menu,
    MoreHorizontal,
    MoreVertical,
    ExternalLink,
    Compass,
    MapPin,

    // Actions
    Plus,
    Minus,
    Check,
    X,
    Trash,
    Trash2,
    Edit,
    Edit2,
    Copy,
    Save,
    Share,
    Download,
    Upload,
    RefreshCw,
    RefreshCcw,
    Filter,
    ZoomIn,
    ZoomOut,
    Maximize,
    Minimize,

    // Media
    Play,
    Pause,
    Square,
    SkipForward,
    SkipBack,
    Volume,
    Volume1,
    Volume2,
    VolumeX,
    Mic,
    MicOff,
    Video,
    VideoOff,
    Camera,
    Image,
    Music,

    // Devices & Hardware
    Laptop,
    Monitor,
    Smartphone,
    Tablet,
    Tv,
    HardDrive,
    Cpu,
    Battery,
    BatteryCharging,
    Wifi,
    WifiOff,
    Bluetooth,
    Server,
    Database,
    Terminal,

    // Communication
    Mail,
    Inbox,
    Send,
    AtSign,
    MessageSquare,
    MessageCircle,
    Phone,
    PhoneCall,
    PhoneOff,
    Bell,
    BellOff,
    Globe,

    // Editor & Text
    Bold,
    Italic,
    Underline,
    AlignLeft,
    AlignCenter,
    AlignRight,
    AlignJustify,
    List,
    Code,
    Link,
    Link2,
    Scissors,
    Hash,

    // System & Status
    Search,
    Settings,
    Tool,
    User,
    Users,
    UserPlus,
    UserCheck,
    UserX,
    Lock,
    Unlock,
    Key,
    Shield,
    ShieldCheck,
    Eye,
    EyeOff,
    Sun,
    Moon,
    Info,
    HelpCircle,
    AlertCircle,
    AlertTriangle,
    CheckCircle,
    XCircle,
    Clock,
    Calendar,
    Bookmark,
    Star,
    Heart,
    Flag,
}

impl LucideIcon {
    /// Returns the SVG vector path data in 24x24 coordinate space.
    #[must_use]
    pub const fn svg_path(self) -> &'static str {
        if let Some(path) = self.nav_or_action_svg_path() {
            return path;
        }
        if let Some(path) = self.media_device_comm_svg_path() {
            return path;
        }
        self.editor_or_system_svg_path()
    }

    const fn nav_or_action_svg_path(self) -> Option<&'static str> {
        Some(match self {
            // Navigation
            Self::ChevronDown => NavigationIcon::ChevronDown.svg_path(),
            Self::ChevronUp => NavigationIcon::ChevronUp.svg_path(),
            Self::ChevronLeft => NavigationIcon::ChevronLeft.svg_path(),
            Self::ChevronRight => NavigationIcon::ChevronRight.svg_path(),
            Self::ChevronsDown => NavigationIcon::ChevronsDown.svg_path(),
            Self::ChevronsUp => NavigationIcon::ChevronsUp.svg_path(),
            Self::ChevronsLeft => NavigationIcon::ChevronsLeft.svg_path(),
            Self::ChevronsRight => NavigationIcon::ChevronsRight.svg_path(),
            Self::ArrowDown => NavigationIcon::ArrowDown.svg_path(),
            Self::ArrowUp => NavigationIcon::ArrowUp.svg_path(),
            Self::ArrowLeft => NavigationIcon::ArrowLeft.svg_path(),
            Self::ArrowRight => NavigationIcon::ArrowRight.svg_path(),
            Self::ArrowUpRight => NavigationIcon::ArrowUpRight.svg_path(),
            Self::ArrowDownRight => NavigationIcon::ArrowDownRight.svg_path(),
            Self::CornerUpLeft => NavigationIcon::CornerUpLeft.svg_path(),
            Self::CornerDownRight => NavigationIcon::CornerDownRight.svg_path(),
            Self::Home => NavigationIcon::Home.svg_path(),
            Self::Menu => NavigationIcon::Menu.svg_path(),
            Self::MoreHorizontal => NavigationIcon::MoreHorizontal.svg_path(),
            Self::MoreVertical => NavigationIcon::MoreVertical.svg_path(),
            Self::ExternalLink => NavigationIcon::ExternalLink.svg_path(),
            Self::Compass => NavigationIcon::Compass.svg_path(),
            Self::MapPin => NavigationIcon::MapPin.svg_path(),

            // Actions
            Self::Plus => ActionIcon::Plus.svg_path(),
            Self::Minus => ActionIcon::Minus.svg_path(),
            Self::Check => ActionIcon::Check.svg_path(),
            Self::X => ActionIcon::X.svg_path(),
            Self::Trash => ActionIcon::Trash.svg_path(),
            Self::Trash2 => ActionIcon::Trash2.svg_path(),
            Self::Edit => ActionIcon::Edit.svg_path(),
            Self::Edit2 => ActionIcon::Edit2.svg_path(),
            Self::Copy => ActionIcon::Copy.svg_path(),
            Self::Save => ActionIcon::Save.svg_path(),
            Self::Share => ActionIcon::Share.svg_path(),
            Self::Download => ActionIcon::Download.svg_path(),
            Self::Upload => ActionIcon::Upload.svg_path(),
            Self::RefreshCw => ActionIcon::RefreshCw.svg_path(),
            Self::RefreshCcw => ActionIcon::RefreshCcw.svg_path(),
            Self::Filter => ActionIcon::Filter.svg_path(),
            Self::ZoomIn => ActionIcon::ZoomIn.svg_path(),
            Self::ZoomOut => ActionIcon::ZoomOut.svg_path(),
            Self::Maximize => ActionIcon::Maximize.svg_path(),
            Self::Minimize => ActionIcon::Minimize.svg_path(),
            _ => return None,
        })
    }

    const fn media_device_comm_svg_path(self) -> Option<&'static str> {
        Some(match self {
            // Media
            Self::Play => MediaIcon::Play.svg_path(),
            Self::Pause => MediaIcon::Pause.svg_path(),
            Self::Square => MediaIcon::Square.svg_path(),
            Self::SkipForward => MediaIcon::SkipForward.svg_path(),
            Self::SkipBack => MediaIcon::SkipBack.svg_path(),
            Self::Volume => MediaIcon::Volume.svg_path(),
            Self::Volume1 => MediaIcon::Volume1.svg_path(),
            Self::Volume2 => MediaIcon::Volume2.svg_path(),
            Self::VolumeX => MediaIcon::VolumeX.svg_path(),
            Self::Mic => MediaIcon::Mic.svg_path(),
            Self::MicOff => MediaIcon::MicOff.svg_path(),
            Self::Video => MediaIcon::Video.svg_path(),
            Self::VideoOff => MediaIcon::VideoOff.svg_path(),
            Self::Camera => MediaIcon::Camera.svg_path(),
            Self::Image => MediaIcon::Image.svg_path(),
            Self::Music => MediaIcon::Music.svg_path(),

            // Devices
            Self::Laptop => DeviceIcon::Laptop.svg_path(),
            Self::Monitor => DeviceIcon::Monitor.svg_path(),
            Self::Smartphone => DeviceIcon::Smartphone.svg_path(),
            Self::Tablet => DeviceIcon::Tablet.svg_path(),
            Self::Tv => DeviceIcon::Tv.svg_path(),
            Self::HardDrive => DeviceIcon::HardDrive.svg_path(),
            Self::Cpu => DeviceIcon::Cpu.svg_path(),
            Self::Battery => DeviceIcon::Battery.svg_path(),
            Self::BatteryCharging => DeviceIcon::BatteryCharging.svg_path(),
            Self::Wifi => DeviceIcon::Wifi.svg_path(),
            Self::WifiOff => DeviceIcon::WifiOff.svg_path(),
            Self::Bluetooth => DeviceIcon::Bluetooth.svg_path(),
            Self::Server => DeviceIcon::Server.svg_path(),
            Self::Database => DeviceIcon::Database.svg_path(),
            Self::Terminal => DeviceIcon::Terminal.svg_path(),

            // Communication
            Self::Mail => CommunicationIcon::Mail.svg_path(),
            Self::Inbox => CommunicationIcon::Inbox.svg_path(),
            Self::Send => CommunicationIcon::Send.svg_path(),
            Self::AtSign => CommunicationIcon::AtSign.svg_path(),
            Self::MessageSquare => CommunicationIcon::MessageSquare.svg_path(),
            Self::MessageCircle => CommunicationIcon::MessageCircle.svg_path(),
            Self::Phone => CommunicationIcon::Phone.svg_path(),
            Self::PhoneCall => CommunicationIcon::PhoneCall.svg_path(),
            Self::PhoneOff => CommunicationIcon::PhoneOff.svg_path(),
            Self::Bell => CommunicationIcon::Bell.svg_path(),
            Self::BellOff => CommunicationIcon::BellOff.svg_path(),
            Self::Globe => CommunicationIcon::Globe.svg_path(),
            _ => return None,
        })
    }

    const fn editor_or_system_svg_path(self) -> &'static str {
        match self {
            // Editor
            Self::Bold => EditorIcon::Bold.svg_path(),
            Self::Italic => EditorIcon::Italic.svg_path(),
            Self::Underline => EditorIcon::Underline.svg_path(),
            Self::AlignLeft => EditorIcon::AlignLeft.svg_path(),
            Self::AlignCenter => EditorIcon::AlignCenter.svg_path(),
            Self::AlignRight => EditorIcon::AlignRight.svg_path(),
            Self::AlignJustify => EditorIcon::AlignJustify.svg_path(),
            Self::List => EditorIcon::List.svg_path(),
            Self::Code => EditorIcon::Code.svg_path(),
            Self::Link => EditorIcon::Link.svg_path(),
            Self::Link2 => EditorIcon::Link2.svg_path(),
            Self::Scissors => EditorIcon::Scissors.svg_path(),
            Self::Hash => EditorIcon::Hash.svg_path(),

            // System
            Self::Search => SystemIcon::Search.svg_path(),
            Self::Settings => SystemIcon::Settings.svg_path(),
            Self::Tool => SystemIcon::Tool.svg_path(),
            Self::User => SystemIcon::User.svg_path(),
            Self::Users => SystemIcon::Users.svg_path(),
            Self::UserPlus => SystemIcon::UserPlus.svg_path(),
            Self::UserCheck => SystemIcon::UserCheck.svg_path(),
            Self::UserX => SystemIcon::UserX.svg_path(),
            Self::Lock => SystemIcon::Lock.svg_path(),
            Self::Unlock => SystemIcon::Unlock.svg_path(),
            Self::Key => SystemIcon::Key.svg_path(),
            Self::Shield => SystemIcon::Shield.svg_path(),
            Self::ShieldCheck => SystemIcon::ShieldCheck.svg_path(),
            Self::Eye => SystemIcon::Eye.svg_path(),
            Self::EyeOff => SystemIcon::EyeOff.svg_path(),
            Self::Sun => SystemIcon::Sun.svg_path(),
            Self::Moon => SystemIcon::Moon.svg_path(),
            Self::Info => SystemIcon::Info.svg_path(),
            Self::HelpCircle => SystemIcon::HelpCircle.svg_path(),
            Self::AlertCircle => SystemIcon::AlertCircle.svg_path(),
            Self::AlertTriangle => SystemIcon::AlertTriangle.svg_path(),
            Self::CheckCircle => SystemIcon::CheckCircle.svg_path(),
            Self::XCircle => SystemIcon::XCircle.svg_path(),
            Self::Clock => SystemIcon::Clock.svg_path(),
            Self::Calendar => SystemIcon::Calendar.svg_path(),
            Self::Bookmark => SystemIcon::Bookmark.svg_path(),
            Self::Star => SystemIcon::Star.svg_path(),
            Self::Heart => SystemIcon::Heart.svg_path(),
            Self::Flag => SystemIcon::Flag.svg_path(),
            _ => "",
        }
    }

    /// Converts this icon into a cached VectorPath.
    #[must_use]
    pub fn to_vector_path(self) -> VectorPath {
        VectorPath::parse_svg(self.svg_path())
    }
}
