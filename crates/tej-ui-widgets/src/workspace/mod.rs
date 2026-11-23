//! Workspace, Planning & Productivity Components.
//!
//! Provides production-ready widgets for BSP Docking Panels, zero-copy Kanban Boards,
//! Gantt Charts with dependency linkages, Node Graph Editors, and Timeline Track Editors.

pub mod dock;
pub mod gantt;
pub mod kanban;
pub mod node_editor;
pub mod timeline;

pub use dock::{DockManager, DockManagerState, DockNode, DockSplitOrientation, DockTab};
pub use gantt::{GanttChart, GanttState, GanttTask};
pub use kanban::{KanbanBoard, KanbanCard, KanbanColumn, KanbanState};
pub use node_editor::{GraphNode, NodeConnection, NodeEditor, NodeEditorState, NodePin};
pub use timeline::{Keyframe, TimelineEditor, TimelineState, TimelineTrack};
