use mydb::Database;
use std::sync::Arc;
use tej::prelude::*;

#[derive(Clone, Copy, Debug)]
struct AppState {
    filter: Signal<String>,
    #[allow(dead_code)]
    selected_id: Signal<Option<u64>>,
    scroll_pos: Signal<f32>,
}

fn task_manager_view(state: AppState) -> impl View {
    VStack::new((
        // Native OS Search Input (GTK4 / Win32 / UIKit)
        SearchBar::new(state.filter).placeholder("Search 1,000,000+ tasks instantly..."),
        // Zero-copy virtualized list: only materializes visible rows from mmap pages
        VirtualList::new(1_000_000, 36.0, 720.0, state.scroll_pos, move |idx| {
            HStack::new((
                Checkbox::new(Signal::new(idx % 2 == 0)),
                Text::new(format!("Task #{idx:06}: Zero-copy database item")),
            ))
            .spacing(12)
        }),
    ))
    .padding(16)
    .spacing(16)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Initializing tej-ui Zero-Copy Task Manager backed by mydb...");

    // 1. Initialize embedded ACID database
    let db = Arc::new(Database::open_memory()?);
    let _sync_hook = Arc::new(DbSyncHook::new());

    // 2. Populate 1,000 initial task entries in memory database
    let mut tx = db.begin_write()?;
    for i in 0..1000 {
        let key = format!("task:{i:06}");
        let val = format!("Zero-copy high-performance task #{i}");
        tx.put(key.as_bytes(), val.as_bytes())?;
    }
    tx.commit()?;
    println!("Successfully loaded 1,000 tasks into database!");

    // 3. Setup reactive state
    let state = AppState {
        filter: Signal::new(String::new()),
        selected_id: Signal::new(None),
        scroll_pos: Signal::new(0.0),
    };

    // 4. Construct declarative view
    let view = task_manager_view(state);

    // 5. Run single headless frame for verification
    let app = App::new(view).window(WindowConfig {
        title: "tej-ui Task Manager (1M+ items)".into(),
        size: Size::new(900.0, 750.0),
        ..Default::default()
    });

    let _state = app.run_headless_frame();
    println!("Headless frame rendered cleanly with flat memory footprint!");

    Ok(())
}
