//! Semantic accessibility roles and actions according to platform assistive tech standards.

/// Semantic accessibility roles matching platform assistive tech standards.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
pub enum A11yRole {
    #[default]
    Generic,
    Window,
    Button,
    Checkbox,
    TextInput,
    Label,
    List,
    ListItem,
    ScrollArea,
    Slider,
    Switch,
    RadioButton,
    Header,
    Link,
    Image,
    Dialog,
    ProgressBar,
    Tab,
    TabList,
    Menu,
    MenuItem,
    Tree,
    TreeItem,
    Grid,
    ComboBox,
    Alert,
    Tooltip,
}

impl From<A11yRole> for accesskit::Role {
    fn from(role: A11yRole) -> Self {
        match role {
            A11yRole::Generic => Self::GenericContainer,
            A11yRole::Window => Self::Window,
            A11yRole::Button => Self::Button,
            A11yRole::Checkbox => Self::CheckBox,
            A11yRole::TextInput => Self::TextInput,
            A11yRole::Label => Self::Label,
            A11yRole::List => Self::List,
            A11yRole::ListItem => Self::ListItem,
            A11yRole::ScrollArea => Self::ScrollView,
            A11yRole::Slider => Self::Slider,
            A11yRole::Switch => Self::Switch,
            A11yRole::RadioButton => Self::RadioButton,
            A11yRole::Header => Self::Heading,
            A11yRole::Link => Self::Link,
            A11yRole::Image => Self::Image,
            A11yRole::Dialog => Self::Dialog,
            A11yRole::ProgressBar => Self::ProgressIndicator,
            A11yRole::Tab => Self::Tab,
            A11yRole::TabList => Self::TabList,
            A11yRole::Menu => Self::Menu,
            A11yRole::MenuItem => Self::MenuItem,
            A11yRole::Tree => Self::Tree,
            A11yRole::TreeItem => Self::TreeItem,
            A11yRole::Grid => Self::Grid,
            A11yRole::ComboBox => Self::ComboBox,
            A11yRole::Alert => Self::Alert,
            A11yRole::Tooltip => Self::Tooltip,
        }
    }
}

/// Actions invoked by platform assistive tech (screen readers, voice control).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum A11yAction {
    Click,
    Focus,
    Toggle,
    ScrollForward,
    ScrollBackward,
}

impl From<A11yAction> for accesskit::Action {
    fn from(action: A11yAction) -> Self {
        match action {
            A11yAction::Click | A11yAction::Toggle => Self::Click,
            A11yAction::Focus => Self::Focus,
            A11yAction::ScrollForward => Self::ScrollDown,
            A11yAction::ScrollBackward => Self::ScrollUp,
        }
    }
}
