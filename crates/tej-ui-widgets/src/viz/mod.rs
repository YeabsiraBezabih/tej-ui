//! Advanced Data Visualizations & Analytics Suite.
//!
//! Provides production-ready widgets for 52-week Calendar Heatmaps, Sankey Flow Diagrams,
//! Squarified Treemaps, Activity Rings, Candlestick charts, and Dot-Matrix Displays.

pub mod charts_extended;
pub mod gauges;
pub mod heatmap;
pub mod sankey;
pub mod treemap;

pub use charts_extended::{
    CandlestickChart, CandlestickPoint, CandlestickState, DonutSlice, FunnelStage,
};
pub use gauges::{ActivityRingsState, DotMatrixDisplay, RadialGauge, RingMetric};
pub use heatmap::{CalendarHeatmap, CalendarHeatmapState, HeatmapCell};
pub use sankey::{SankeyDiagram, SankeyLink, SankeyNode, SankeyState};
pub use treemap::{Treemap, TreemapNode, TreemapState};
