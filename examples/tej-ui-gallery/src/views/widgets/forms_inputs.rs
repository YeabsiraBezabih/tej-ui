//! Forms and input controls showcase view.

use tej_ui::prelude::*;

/// Category view showcasing text inputs, search, switches, sliders, checkboxes, and radio groups.
pub fn forms_inputs_view() -> impl View {
    let input_text = Signal::new("Hello Tej UI".to_string());
    let search_text = Signal::new(String::new());
    let check_val = Signal::new(true);
    let switch_val = Signal::new(true);
    let slider_val = Signal::new(65.0f32);
    let radio_selected = Signal::new("Standard".to_string());
    let select_selected = Signal::new("US-East".to_string());
    let select_open = Signal::new(false);

    let radio_items = vec![
        RadioItem::new("Standard".to_string(), "Standard Cloud"),
        RadioItem::new("Dedicated".to_string(), "Dedicated Bare-Metal"),
        RadioItem::new("Edge".to_string(), "Edge Wasm Nodes"),
    ];

    let select_options = vec![
        SelectOption::new("US-East".to_string(), "US-East (N. Virginia)"),
        SelectOption::new("EU-Central".to_string(), "EU-Central (Frankfurt)"),
        SelectOption::new("AP-South".to_string(), "AP-South (Mumbai)"),
    ];

    VStack::new((
        // Section 1: Text Fields & Search
        Card::new("1. Text Input & Search Fields")
            .description(
                "Focus-managed single-line inputs with keyboard typing and backspace support",
            )
            .flex_grow(1.0)
            .content(
                HStack::new((
                    VStack::new((
                        Text::new("Interactive Text Input"),
                        Input::new(input_text).placeholder("Type text here..."),
                    ))
                    .spacing(4),
                    VStack::new((
                        Text::new("Search Bar"),
                        SearchBar::new(search_text).placeholder("Search symbols or widgets..."),
                    ))
                    .spacing(4),
                ))
                .spacing(24),
            ),
        // Section 2: Toggles & Range Controls
        Card::new("2. Sliders, Switches & Checkboxes")
            .description("Kinetic continuous ranges and accessible boolean toggles")
            .flex_grow(1.0)
            .content(
                HStack::new((
                    VStack::new((
                        Text::new("Slider Range (65%)"),
                        Slider::new(slider_val, 0.0, 100.0),
                    ))
                    .spacing(4),
                    VStack::new((Text::new("Toggle Switch"), Switch::new(switch_val))).spacing(4),
                    VStack::new((Text::new("Checkbox"), Checkbox::new(check_val))).spacing(4),
                ))
                .spacing(32),
            ),
        // Section 3: Single Selection Controls
        Card::new("3. Radio Groups & Select Pickers")
            .description("Single-choice list selection and dropdown select controls")
            .flex_grow(1.0)
            .content(
                HStack::new((
                    VStack::new((
                        Text::new("Deployment Tier"),
                        RadioGroup::new(radio_selected, radio_items),
                    ))
                    .spacing(4),
                    VStack::new((
                        Text::new("Target Cloud Region"),
                        Select::new(
                            select_selected,
                            select_open,
                            select_options,
                            "Select region...",
                        ),
                    ))
                    .spacing(4),
                ))
                .spacing(32),
            ),
    ))
    .spacing(16)
}
