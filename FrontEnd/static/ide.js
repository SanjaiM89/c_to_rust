// Monaco Editor Controller for c_to_rust Web VS Code IDE

let editor = null;
let currentFile = 'src/lib.rs';
let activeModel = null;
let isStreamingActive = false;
let eventSource = null;
let currentPanelTab = 'terminal';

// Configure Monaco Editor loader
require.config({ paths: { 'vs': 'https://cdnjs.cloudflare.com/ajax/libs/monaco-editor/0.45.0/min/vs' } });

window.addEventListener('DOMContentLoaded', () => {
  require(['vs/editor/editor.main'], function () {
    // Define VS Code Dark+ theme
    monaco.editor.setTheme('vs-dark');

    editor = monaco.editor.create(document.getElementById('monaco-container'), {
      value: '// Initializing workspace...\n',
      language: 'rust',
      theme: 'vs-dark',
      automaticLayout: true,
      fontSize: 14,
      fontFamily: "'JetBrains Mono', 'Fira Code', monospace",
      minimap: { enabled: true },
      scrollBeyondLastLine: false,
      renderWhitespace: 'selection',
      cursorBlinking: 'smooth',
      smoothScrolling: true,
      padding: { top: 12, bottom: 12 }
    });

    editor.onDidChangeCursorPosition((e) => {
      document.getElementById('status-cursor').textContent = `Ln ${e.position.lineNumber}, Col ${e.position.column}`;
    });

    loadFileTree();
    openFile('src/lib.rs');
    initIdeSSE();
  });
});

async function loadFileTree() {
  const treeContainer = document.getElementById('file-tree');
  try {
    const res = await fetch('/api/files');
    if (!res.ok) {
      treeContainer.innerHTML = '<div style="padding: 1rem; color: #EF4444;">Failed to load files</div>';
      return;
    }
    const data = await res.json();
    document.getElementById('sidebar-root-name').textContent = data.root.toUpperCase();
    document.getElementById('active-crate-tag').textContent = `Active Crate: ${data.root}`;

    treeContainer.innerHTML = '';
    data.files.forEach(file => {
      const item = document.createElement('div');
      item.className = `ide-file-item ${file.path === currentFile ? 'active' : ''}`;
      item.onclick = () => openFile(file.path);

      let icon = '📄';
      if (file.language === 'rust') icon = '🦀';
      else if (file.language === 'c') icon = '©️';
      else if (file.language === 'toml') icon = '⚙️';
      else if (file.language === 'markdown') icon = '📝';

      item.innerHTML = `
        <span>${icon}</span>
        <span style="flex: 1; overflow: hidden; text-overflow: ellipsis; white-space: nowrap;">${file.path}</span>
        <span style="font-size: 0.7rem; color: #6E7681;">${(file.size / 1024).toFixed(1)}k</span>
      `;
      treeContainer.appendChild(item);
    });
  } catch (err) {
    treeContainer.innerHTML = `<div style="padding: 1rem; color: #EF4444;">Error: ${err.message}</div>`;
  }
}

async function openFile(relPath) {
  currentFile = relPath;
  document.getElementById('tab-filename').textContent = relPath;

  // Update active file class in sidebar
  document.querySelectorAll('.ide-file-item').forEach(el => {
    if (el.textContent.includes(relPath)) {
      el.classList.add('active');
    } else {
      el.classList.remove('active');
    }
  });

  try {
    const res = await fetch(`/api/file?path=${encodeURIComponent(relPath)}`);
    if (!res.ok) {
      if (!isStreamingActive) {
        editor.setValue(`// File: ${relPath}\n// Waiting for synthesis to generate contents...`);
      }
      return;
    }
    const data = await res.json();
    
    // Determine language
    let lang = 'plaintext';
    if (relPath.endsWith('.rs')) lang = 'rust';
    else if (relPath.endsWith('.c') || relPath.endsWith('.h')) lang = 'c';
    else if (relPath.endsWith('.toml')) lang = 'ini';
    else if (relPath.endsWith('.md')) lang = 'markdown';
    else if (relPath.endsWith('.json')) lang = 'json';

    document.getElementById('status-lang').textContent = lang.toUpperCase();

    const oldModel = editor.getModel();
    const newModel = monaco.editor.createModel(data.content, lang);
    editor.setModel(newModel);
    if (oldModel) oldModel.dispose();

  } catch (err) {
    if (!isStreamingActive) {
      editor.setValue(`// Error loading file: ${err.message}`);
    }
  }
}

// Integrated Terminal in Web VS Code
function appendIdeTerminal(text, type = 'normal') {
  const terminal = document.getElementById('ide-terminal-body');
  if (!terminal) return;

  const line = document.createElement('div');
  line.className = 'terminal-line';

  const prefix = document.createElement('span');
  prefix.className = 'terminal-cmd';
  prefix.textContent = type === 'cmd' ? '$' : '>';

  const content = document.createElement('span');
  if (type === 'cmd') content.className = 'terminal-cmd';
  else if (type === 'success') content.className = 'terminal-success';
  else if (type === 'warn') content.className = 'terminal-warn';
  else if (type === 'error') content.className = 'terminal-error';

  content.textContent = text;
  line.appendChild(prefix);
  line.appendChild(content);

  terminal.appendChild(line);
  terminal.scrollTop = terminal.scrollHeight;
}

function clearIdeTerminal() {
  const terminal = document.getElementById('ide-terminal-body');
  if (terminal) {
    terminal.innerHTML = '<div class="terminal-line"><span class="terminal-cmd">$</span> <span>Terminal cleared.</span></div>';
  }
}

function toggleIdePanel() {
  const panel = document.getElementById('ide-bottom-panel');
  if (panel) {
    panel.classList.toggle('collapsed');
    if (editor) editor.layout();
  }
}

function switchIdePanel(tab) {
  currentPanelTab = tab;
  ['terminal', 'output', 'problems'].forEach(t => {
    const btn = document.getElementById(`tab-btn-${t}`);
    if (btn) {
      if (t === tab) btn.classList.add('active');
      else btn.classList.remove('active');
    }
  });
}

// Real-Time SSE Listener for Web VS Code
function initIdeSSE() {
  if (eventSource) {
    eventSource.close();
  }

  eventSource = new EventSource('/api/events');

  eventSource.addEventListener('init', (e) => {
    const data = JSON.parse(e.data);
    if (data.is_running) {
      showStreamingIndicator('Translating active...');
      focusTranslatingFile();
    }
  });

  eventSource.addEventListener('status', (e) => {
    const data = JSON.parse(e.data);
    if (data.state === 'started') {
      isStreamingActive = true;
      showStreamingIndicator(`Translating ${data.source || 'repository'}...`);
      focusTranslatingFile();
      appendIdeTerminal(`Starting translation for ${data.source}...`, 'cmd');
    } else if (data.state === 'idle') {
      isStreamingActive = false;
      hideStreamingIndicator();
      loadFileTree();
      if (data.cancelled) {
        appendIdeTerminal('🛑 Process cancelled.', 'warn');
        document.getElementById('status-safety').textContent = 'Translation Stopped';
      }
    }
  });

  eventSource.addEventListener('stage', (e) => {
    const data = JSON.parse(e.data);
    showStreamingIndicator(`[Stage ${data.id}] ${data.name}`);
    appendIdeTerminal(`[Stage ${data.id}] ${data.name}: ${data.desc}`, 'cmd');

    if (data.id === 2 || data.id === 3 || data.id === 5) {
      loadFileTree();
    }
    if (data.id === 3) {
      focusTranslatingFile();
    }
  });

  eventSource.addEventListener('command', (e) => {
    const data = JSON.parse(e.data);
    const statusText = data.status ? ` [${data.status.toUpperCase()}]` : '';
    appendIdeTerminal(`${data.cmd}${statusText}`, data.status === 'passed' ? 'success' : 'cmd');

    if (data.cmd.includes('cargo check')) {
      document.getElementById('status-safety').textContent = 'Running cargo check...';
    } else if (data.cmd.includes('cargo test')) {
      document.getElementById('status-safety').textContent = 'Running cargo test...';
    }
  });

  eventSource.addEventListener('log', (e) => {
    const data = JSON.parse(e.data);
    const text = data.text;
    if (text.includes('PASSED')) {
      appendIdeTerminal(text, 'success');
    } else if (text.includes('Warning') || text.includes('Repair')) {
      appendIdeTerminal(text, 'warn');
    } else if (text.includes('failed') || text.includes('Error')) {
      appendIdeTerminal(text, 'error');
    } else {
      appendIdeTerminal(text, 'normal');
    }
  });

  // Live Token-by-Token Streaming inside Monaco Editor!
  eventSource.addEventListener('rust_token', (e) => {
    const data = JSON.parse(e.data);
    if (!editor) return;

    // Automatically focus on src/lib.rs if not already
    if (currentFile !== 'src/lib.rs') {
      currentFile = 'src/lib.rs';
      document.getElementById('tab-filename').textContent = '● src/lib.rs [Generating...]';
      const model = monaco.editor.createModel('', 'rust');
      editor.setModel(model);
    } else {
      document.getElementById('tab-filename').textContent = '● src/lib.rs [Generating...]';
    }

    const model = editor.getModel();
    if (model) {
      const lineCount = model.getLineCount();
      const lastLineLength = model.getLineMaxColumn(lineCount);
      const range = new monaco.Range(lineCount, lastLineLength, lineCount, lastLineLength);
      model.applyEdits([{ range: range, text: data.token }]);
      editor.revealLine(lineCount);
    }
  });

  eventSource.addEventListener('complete', (e) => {
    const data = JSON.parse(e.data);
    isStreamingActive = false;

    showStreamingIndicator('✓ Translation Complete', true);
    document.getElementById('tab-filename').textContent = 'src/lib.rs';
    document.getElementById('status-safety').textContent = '100% Safe Rust - All Tests Passed';
    appendIdeTerminal('🎉 Crate compilation and test suite passed 100%!', 'success');

    loadFileTree();
  });

  eventSource.addEventListener('error', (e) => {
    try {
      const data = JSON.parse(e.data);
      appendIdeTerminal(`Error: ${data.message}`, 'error');
    } catch (_) {}
    isStreamingActive = false;
    hideStreamingIndicator();
  });
}

function focusTranslatingFile() {
  currentFile = 'src/lib.rs';
  document.getElementById('tab-filename').textContent = '● src/lib.rs [Generating...]';
  if (editor) {
    const currentModel = editor.getModel();
    if (!currentModel || currentModel.getValue().includes('// Initializing') || currentModel.getValue().includes('// Loading')) {
      const newModel = monaco.editor.createModel('// Translating to safe Rust lively...\n', 'rust');
      editor.setModel(newModel);
    }
  }
}

function showStreamingIndicator(text, isComplete = false) {
  const badge = document.getElementById('ide-live-status');
  const txt = document.getElementById('ide-live-status-text');
  if (badge && txt) {
    badge.style.display = 'flex';
    txt.textContent = text;
    if (isComplete) {
      badge.style.borderColor = 'var(--accent-green)';
      badge.style.color = 'var(--accent-green)';
      badge.style.background = 'rgba(16, 185, 129, 0.15)';
    } else {
      badge.style.borderColor = 'var(--primary)';
      badge.style.color = 'var(--primary)';
      badge.style.background = 'rgba(6, 182, 212, 0.2)';
    }
  }
}

function hideStreamingIndicator() {
  const badge = document.getElementById('ide-live-status');
  if (badge) {
    setTimeout(() => {
      badge.style.display = 'none';
    }, 4000);
  }
}
