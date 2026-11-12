/// Semantic accessibility roles according with platform assistive tech standards.
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
