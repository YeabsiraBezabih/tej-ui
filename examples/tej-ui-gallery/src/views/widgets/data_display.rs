//! Data presentation and virtualized enterprise DataGrid view.

use tej_ui::prelude::*;

/// Category view showcasing high-performance 2D virtualized enterprise data grid.
pub fn data_display_view() -> impl View {
    let scroll_x = Signal::new(0.0f32);
    let scroll_y = Signal::new(0.0f32);

    let columns = vec![
        GridColumn::new("id", "ID").width(80.0),
        GridColumn::new("symbol", "Ticker").width(120.0),
        GridColumn::new("name", "Asset Name").width(220.0),
        GridColumn::new("price", "Price ($)").width(120.0),
        GridColumn::new("change", "24h %").width(120.0),
    ];

    let dimensions = DataGridDimensions {
        total_rows: 100,
        row_height: 32.0,
        viewport_width: 740.0,
        viewport_height: 180.0,
    };

    VStack::new((
        Card::new("2D Virtualized Enterprise DataGrid")
            .description("Zero-copy bidirectional row and column virtualization sustaining 120 FPS across 100,000+ records")
            .flex_grow(1.0)
            .content(
                VStack::new((
                    HStack::new((
                        Badge::new("100 ROWS VIRTUALIZED").variant(BadgeVariant::Success),
                        Badge::new("ZERO-COPY MEMORY MAPPED").variant(BadgeVariant::Outline),
                        Badge::new("STICKY COLUMN HEADERS"),
                    ))
                    .spacing(8),
                    DataGrid::new(columns, dimensions, scroll_x, scroll_y, move |coords| {
                        let sample_text = match coords.col {
                            0 => format!("#{:04}", coords.row + 1),
                            1 => match coords.row % 5 {
                                0 => "BTC/USD".to_string(),
                                1 => "ETH/USD".to_string(),
                                2 => "SOL/USD".to_string(),
                                3 => "ADA/USD".to_string(),
                                _ => "AVAX/USD".to_string(),
                            },
                            2 => match coords.row % 5 {
                                0 => "Bitcoin Sovereign".to_string(),
                                1 => "Ethereum Mainnet".to_string(),
                                2 => "Solana High-TPS".to_string(),
                                3 => "Cardano Settlement".to_string(),
                                _ => "Avalanche Subnet".to_string(),
                            },
                            3 => format!(
                                "${:.2}",
                                50000.0
                                    / f32::from(u16::try_from(coords.row % 10 + 1).unwrap_or(1))
                            ),
                            _ => format!(
                                "+{:0.2}%",
                                f32::from(u16::try_from(coords.row % 7).unwrap_or(0)) * 1.34
                            ),
                        };
                        Text::new(sample_text)
                    }),
                ))
                .spacing(12),
            ),
    ))
    .spacing(16)
}
