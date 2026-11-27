# Widgets and Forms (`tej-ui-view` & `tej-ui-widgets`)

`tej-ui` splits UI building blocks into two layers:
1. **Core Primitives (`tej-ui-view`)** — always available, even with `default-features = false`.
2. **Enterprise Widget Library (`tej-ui-widgets`)** — enabled by default via the `widgets` feature flag.

## 1. Core View Primitives (`tej-ui-view`)

| Primitive | Purpose |
| :--- | :--- |
| `Text` | Static (`Text::new("...")`) or reactive (`Text::reactive(signal)`) text with optional `.color(Color)`. |
| `Button<V>` | Interactive button wrapping any child `View`: `Button::new(Text::new("Save")).variant(ButtonVariant::Default).size(ButtonSize::Default).on_click(...)`. |
| `Checkbox` | Boolean toggle bound to `Signal<bool>`. |
| `Input` | Single-line text input bound to `Signal<String>`. |
| `SearchBar` | Search input with `.placeholder(...)` bound to `Signal<String>`. |
| `VStack`, `HStack`, `Grid` | Static tuple layout containers. |
| `Canvas` | Retained 2D drawing surface receiving `&mut dyn SurfaceCompositor` and `Rect` bounds. |
| `If`, `For` | Conditional (`If::new(cond_signal, then_view, else_view)`) and keyed list (`For::new(items, key_fn, view_fn)`) control-flow views. |

## 2. Enterprise Widgets (`tej-ui-widgets`)

### Data Display & Navigation
- **`DataGrid`**: 2D virtualized data table with `GridColumn`, `GridCellCoords`, `DataGridDimensions`, `SortDirection`, and `PinState`.
- **`TreeView` & `TreeNode`**: Hierarchical collapsible tree navigation.
- **`Tabs` / `TabItem`**, **`Accordion` / `AccordionItem`**, **`Sidebar` / `SidebarItem`**, **`Breadcrumb` / `BreadcrumbItem`**, **`Pagination`**, **`ScrollArea`**, **`Separator`**.
- **`Card`**, **`Badge`** (`BadgeVariant`), **`Avatar`**, **`Skeleton`**, **`Progress`**.

### Rich Editing & Command Surfaces
- **`RichTextEditor` & `RichTextEditorState`**: Multi-line editing with `TextBuffer`, `TextSelection`, `CursorPos`, `EditAction`, `UndoStack`, and `ImeState`.
- **`CommandPalette`**: Keyboard-driven command launcher with `CommandItem`, `CommandAction`, and `fuzzy_match`.

### Inputs & Overlays
- **`Select` / `SelectOption`**, **`RadioGroup` / `RadioItem`**, **`Switch`**, **`Slider`**.
- **`Modal`**, **`Sheet`** (`SheetSide`), **`Popover`** (`PopoverPlacement`), **`Tooltip`**, **`Alert`** (`AlertSeverity`), **`ToastItem`** (`ToastSeverity`).

## 3. Forms & Validation (`FormController`)

`tej-ui-widgets` includes a typed form engine (`FormController`, `FormField`) with built-in composable validators exported in `tej_ui::prelude::*`:

- `required` / `required_option`
- `min_length` / `max_length`
- `email`
- `numeric_range`

```rust
use tej_ui::prelude::*;

let email_field = FormField::new(String::new())
    .with_validator(required("Email is required"))
    .with_validator(email("Enter a valid email address"));

let name_field = FormField::new(String::new())
    .with_validator(required("Name is required"))
    .with_validator(min_length(2, "Name must be at least 2 characters"));
```

## 4. Exploring the Widget Gallery

The repository includes an interactive gallery showcasing widgets, CSS properties, accessibility inspection, vector demos, and 3D viewports:

```bash
cargo run -p tej-ui-gallery
```
