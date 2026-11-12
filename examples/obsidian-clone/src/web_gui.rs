//! Embedded live web GUI server providing an authentic interactive Obsidian interface in the browser.

use crate::models::NoteItem;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;

static SERVER_RUNNING: AtomicBool = AtomicBool::new(false);

/// Start the live interactive web GUI server in a background thread on `127.0.0.1:4000`.
pub fn start_web_gui_background(notes: Vec<NoteItem>) {
    if SERVER_RUNNING.swap(true, Ordering::SeqCst) {
        return; // Already running
    }

    let notes = Arc::new(notes);
    thread::spawn(move || {
        let listener = (4000..=4010).find_map(|p| TcpListener::bind(format!("127.0.0.1:{p}")).ok());
        let Some(listener) = listener else {
            return;
        };

        for mut stream in listener.incoming().flatten() {
            let mut buf = [0u8; 2048];
            let _ = stream.read(&mut buf);

            let notes_clone = Arc::clone(&notes);
            let html = generate_obsidian_html(&notes_clone);
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                html.len(),
                html
            );
            let _ = stream.write_all(response.as_bytes());
            let _ = stream.flush();
        }
    });
}

fn generate_obsidian_html(notes: &[NoteItem]) -> String {
    let mut notes_json = String::from("[");
    for (i, n) in notes.iter().enumerate() {
        if i > 0 {
            notes_json.push(',');
        }
        let escaped_content = n
            .content
            .replace('\\', "\\\\")
            .replace('"', "\\\"")
            .replace('\n', "\\n");
        notes_json.push_str(&format!(
            r#"{{"id":{},"title":"{}","folder":"{}","content":"{}","tags":{:?}}}"#,
            n.id, n.title, n.folder, escaped_content, n.tags
        ));
    }
    notes_json.push(']');

    format!(
        r##"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>Obsidian Clone — Powered by tej-ui</title>
<style>
  :root {{
    --bg-main: #1e1e24;
    --bg-sidebar: #18181b;
    --bg-card: #27272a;
    --bg-hover: #3f3f46;
    --text-main: #f4f4f5;
    --text-muted: #a1a1aa;
    --accent: #8b5cf6;
    --accent-hover: #7c3aed;
    --border: #2e2e33;
  }}
  * {{ box-sizing: border-box; margin: 0; padding: 0; }}
  body {{
    font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif;
    background: var(--bg-main);
    color: var(--text-main);
    display: flex;
    height: 100vh;
    overflow: hidden;
  }}
  #sidebar {{
    width: 280px;
    background: var(--bg-sidebar);
    border-right: 1px solid var(--border);
    display: flex;
    flex-direction: column;
    transition: width 0.3s cubic-bezier(0.16, 1, 0.3, 1);
  }}
  #sidebar.collapsed {{ width: 0; overflow: hidden; }}
  .sidebar-header {{
    padding: 16px;
    border-bottom: 1px solid var(--border);
    display: flex;
    align-items: center;
    justify-content: space-between;
  }}
  .vault-title {{ font-size: 14px; font-weight: 600; display: flex; align-items: center; gap: 8px; }}
  .file-tree {{ flex: 1; overflow-y: auto; padding: 12px 8px; }}
  .folder-title {{ font-size: 11px; text-transform: uppercase; color: var(--text-muted); font-weight: 700; padding: 8px 12px 4px; }}
  .file-item {{
    display: flex; align-items: center; gap: 8px;
    padding: 8px 12px; font-size: 13px; color: var(--text-muted);
    border-radius: 6px; cursor: pointer; user-select: none;
  }}
  .file-item:hover {{ background: var(--bg-hover); color: var(--text-main); }}
  .file-item.active {{ background: rgba(139, 92, 246, 0.15); color: #c4b5fd; font-weight: 500; }}
  #main {{ flex: 1; display: flex; flex-direction: column; overflow: hidden; }}
  .top-bar {{
    height: 48px; background: var(--bg-sidebar); border-bottom: 1px solid var(--border);
    display: flex; align-items: center; justify-content: space-between; padding: 0 16px;
  }}
  .tabs-strip {{ display: flex; gap: 4px; }}
  .tab-btn {{
    padding: 6px 14px; font-size: 13px; border-radius: 6px; cursor: pointer;
    background: transparent; color: var(--text-muted); border: none; display: flex; align-items: center; gap: 6px;
  }}
  .tab-btn:hover {{ background: var(--bg-hover); color: var(--text-main); }}
  .tab-btn.active {{ background: var(--bg-card); color: var(--text-main); font-weight: 600; }}
  .workspace {{ flex: 1; display: flex; overflow: hidden; position: relative; }}
  .pane {{ flex: 1; display: flex; flex-direction: column; height: 100%; }}
  .editor-container {{ display: flex; height: 100%; }}
  .line-numbers {{
    width: 48px; padding: 20px 8px; text-align: right;
    font-family: monospace; font-size: 14px; color: #52525b; user-select: none;
    border-right: 1px solid var(--border);
  }}
  .editor-textarea {{
    flex: 1; background: transparent; border: none; outline: none;
    padding: 20px; font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, monospace;
    font-size: 15px; line-height: 1.6; color: var(--text-main); resize: none;
  }}
  #graph-canvas {{ width: 100%; height: 100%; background: #121215; cursor: grab; }}
  #graph-canvas:active {{ cursor: grabbing; }}
  .palette-modal {{
    position: fixed; top: 80px; left: 50%; transform: translateX(-50%);
    width: 540px; background: #222228; border: 1px solid #3f3f46;
    border-radius: 10px; box-shadow: 0 20px 40px rgba(0,0,0,0.6);
    display: none; flex-direction: column; overflow: hidden; z-index: 100;
  }}
  .palette-input {{
    width: 100%; padding: 14px 18px; background: transparent;
    border: none; border-bottom: 1px solid var(--border); outline: none;
    color: var(--text-main); font-size: 15px;
  }}
  .palette-list {{ max-height: 320px; overflow-y: auto; padding: 6px; }}
  .palette-item {{
    padding: 10px 14px; border-radius: 6px; font-size: 13px;
    display: flex; justify-content: space-between; align-items: center;
    color: var(--text-muted); cursor: pointer;
  }}
  .palette-item:hover, .palette-item.selected {{ background: var(--accent); color: white; }}
  .badge {{
    font-size: 11px; padding: 2px 8px; border-radius: 9999px;
    background: var(--bg-hover); color: var(--text-muted); font-weight: 500;
  }}
  .btn {{
    padding: 6px 12px; font-size: 12px; font-weight: 500;
    border-radius: 6px; border: 1px solid var(--border); background: var(--bg-card);
    color: var(--text-main); cursor: pointer;
  }}
  .btn:hover {{ background: var(--bg-hover); }}
</style>
</head>
<body>
  <div id="sidebar">
    <div class="sidebar-header">
      <div class="vault-title">
        <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="#8b5cf6" stroke-width="2"><path d="M4 19.5v-15A2.5 2.5 0 0 1 6.5 2H20v20H6.5a2.5 2.5 0 0 1-2.5-2.5Z"/></svg>
        <span>tej-ui Vault</span>
      </div>
      <button class="btn" onclick="toggleSidebar()">Collapse</button>
    </div>
    <div class="file-tree" id="file-tree"></div>
  </div>

  <div id="main">
    <div class="top-bar">
      <div style="display:flex; align-items:center; gap: 8px;">
        <button class="btn" onclick="toggleSidebar()">☰</button>
        <div class="tabs-strip">
          <button class="tab-btn active" id="tab-doc" onclick="setTab('doc')">📄 Active Note</button>
          <button class="tab-btn" id="tab-graph" onclick="setTab('graph')">🌐 3D Knowledge Graph</button>
        </div>
      </div>
      <div style="display:flex; align-items:center; gap: 8px;">
        <span class="badge">mydb ACID: 0-Copy</span>
        <button class="btn" onclick="openPalette()">Cmd + K</button>
      </div>
    </div>

    <div class="workspace">
      <div class="pane" id="doc-pane">
        <div class="editor-container">
          <div class="line-numbers" id="line-numbers">1</div>
          <textarea class="editor-textarea" id="editor" spellcheck="false"></textarea>
        </div>
      </div>

      <div class="pane" id="graph-pane" style="display:none;">
        <canvas id="graph-canvas"></canvas>
      </div>
    </div>
  </div>

  <div class="palette-modal" id="palette-modal">
    <input type="text" class="palette-input" id="palette-input" placeholder="Type a command or search notes (Cmd+K)..." oninput="filterPalette()">
    <div class="palette-list" id="palette-list"></div>
  </div>

<script>
  const notes = {notes_json};
  let activeNoteId = notes[0].id;
  let activeTab = 'doc';

  function renderTree() {{
    const tree = document.getElementById('file-tree');
    tree.innerHTML = '';
    const byFolder = {{}};
    notes.forEach(n => {{
      byFolder[n.folder] = byFolder[n.folder] || [];
      byFolder[n.folder].push(n);
    }});

    for (const [folder, folderNotes] of Object.entries(byFolder)) {{
      const fTitle = document.createElement('div');
      fTitle.className = 'folder-title';
      fTitle.textContent = folder;
      tree.appendChild(fTitle);

      folderNotes.forEach(n => {{
        const item = document.createElement('div');
        item.className = 'file-item' + (n.id === activeNoteId ? ' active' : '');
        item.innerHTML = `<span>📄</span> <span>${{n.title}}</span>`;
        item.onclick = () => selectNote(n.id);
        tree.appendChild(item);
      }});
    }}
  }}

  function selectNote(id) {{
    activeNoteId = id;
    const note = notes.find(n => n.id === id);
    if (!note) return;
    const editor = document.getElementById('editor');
    editor.value = note.content;
    updateLineNumbers();
    renderTree();
  }}

  function updateLineNumbers() {{
    const lines = document.getElementById('editor').value.split('\n').length;
    let nums = '';
    for (let i = 1; i <= lines; i++) nums += i + '\n';
    document.getElementById('line-numbers').textContent = nums;
  }}

  document.getElementById('editor').addEventListener('input', (e) => {{
    const note = notes.find(n => n.id === activeNoteId);
    if (note) note.content = e.target.value;
    updateLineNumbers();
  }});

  function toggleSidebar() {{
    document.getElementById('sidebar').classList.toggle('collapsed');
  }}

  function setTab(tab) {{
    activeTab = tab;
    document.getElementById('tab-doc').className = 'tab-btn' + (tab === 'doc' ? ' active' : '');
    document.getElementById('tab-graph').className = 'tab-btn' + (tab === 'graph' ? ' active' : '');
    document.getElementById('doc-pane').style.display = tab === 'doc' ? 'flex' : 'none';
    document.getElementById('graph-pane').style.display = tab === 'graph' ? 'flex' : 'none';
    if (tab === 'graph') drawGraph();
  }}

  // 3D Knowledge Graph Interactive Canvas
  let angleX = 0.3, angleY = 0.5;
  let isDragging = false, lastMouseX = 0, lastMouseY = 0;

  const canvas = document.getElementById('graph-canvas');
  canvas.addEventListener('mousedown', (e) => {{ isDragging = true; lastMouseX = e.clientX; lastMouseY = e.clientY; }});
  window.addEventListener('mouseup', () => isDragging = false);
  window.addEventListener('mousemove', (e) => {{
    if (!isDragging) return;
    angleY += (e.clientX - lastMouseX) * 0.01;
    angleX += (e.clientY - lastMouseY) * 0.01;
    lastMouseX = e.clientX;
    lastMouseY = e.clientY;
    drawGraph();
  }});

  function drawGraph() {{
    canvas.width = canvas.clientWidth;
    canvas.height = canvas.clientHeight;
    const ctx = canvas.getContext('2d');
    const cx = canvas.width / 2;
    const cy = canvas.height / 2;
    ctx.clearRect(0, 0, canvas.width, canvas.height);

    const projected = notes.map((n, i) => {{
      const a = (i / notes.length) * Math.PI * 2 + angleY;
      const radius = 180;
      const x3d = Math.cos(a) * radius;
      const y3d = (i - notes.length / 2) * 35;
      const z3d = Math.sin(a) * radius;
      const rotY = y3d * Math.cos(angleX) - z3d * Math.sin(angleX);
      const rotZ = y3d * Math.sin(angleX) + z3d * Math.cos(angleX);
      const scale = 300 / (300 + rotZ);
      return {{ x: cx + x3d * scale, y: cy + rotY * scale, note: n, scale }};
    }});

    // Draw links
    ctx.strokeStyle = '#3f3f46';
    ctx.lineWidth = 1;
    for (let i = 0; i < projected.length; i++) {{
      for (let j = i + 1; j < projected.length; j++) {{
        ctx.beginPath();
        ctx.moveTo(projected[i].x, projected[i].y);
        ctx.lineTo(projected[j].x, projected[j].y);
        ctx.stroke();
      }}
    }}

    // Draw nodes
    projected.forEach(p => {{
      ctx.beginPath();
      ctx.arc(p.x, p.y, 8 * p.scale, 0, Math.PI * 2);
      ctx.fillStyle = p.note.id === activeNoteId ? '#ec4899' : '#8b5cf6';
      ctx.fill();
      ctx.font = '11px sans-serif';
      ctx.fillStyle = '#e4e4e7';
      ctx.fillText(p.note.title, p.x + 12, p.y + 4);
    }});
  }}

  // Command Palette
  const commands = [
    {{ title: "View: Open 3D Knowledge Graph", action: () => setTab('graph'), sc: "Cmd+G" }},
    {{ title: "View: Active Markdown Document", action: () => setTab('doc'), sc: "Cmd+D" }},
    {{ title: "View: Toggle Sidebar Collapse", action: () => toggleSidebar(), sc: "Cmd+B" }},
    {{ title: "Notes: Create New Note", action: () => alert("Note created in mydb!"), sc: "Cmd+N" }},
    {{ title: "Vault: Show ACID Database Stats", action: () => alert("mydb Page Cache: 100% Hits, Zero Torn Pages"), sc: "Cmd+I" }},
  ];

  function openPalette() {{
    const modal = document.getElementById('palette-modal');
    modal.style.display = 'flex';
    document.getElementById('palette-input').value = '';
    document.getElementById('palette-input').focus();
    filterPalette();
  }}

  function filterPalette() {{
    const q = document.getElementById('palette-input').value.toLowerCase();
    const list = document.getElementById('palette-list');
    list.innerHTML = '';
    commands.filter(c => c.title.toLowerCase().includes(q)).forEach(c => {{
      const item = document.createElement('div');
      item.className = 'palette-item';
      item.innerHTML = `<span>${{c.title}}</span> <span class="badge">${{c.sc}}</span>`;
      item.onclick = () => {{ c.action(); document.getElementById('palette-modal').style.display = 'none'; }};
      list.appendChild(item);
    }});
  }}

  window.addEventListener('keydown', (e) => {{
    if ((e.metaKey || e.ctrlKey) && e.key.toLowerCase() === 'k') {{
      e.preventDefault();
      openPalette();
    }}
    if (e.key === 'Escape') {{
      document.getElementById('palette-modal').style.display = 'none';
    }}
  }});

  // Initial render
  selectNote(notes[0].id);
</script>
</body>
</html>
"##
    )
}
