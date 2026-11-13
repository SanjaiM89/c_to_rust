// Monaco Editor Controller for c_to_rust Web VS Code IDE

let editor = null;
let currentFile = 'src/lib.rs';
let activeModel = null;

// Configure Monaco Editor loader
require.config({ paths: { 'vs': 'https://cdnjs.cloudflare.com/ajax/libs/monaco-editor/0.45.0/min/vs' } });

window.addEventListener('DOMContentLoaded', () => {
  require(['vs/editor/editor.main'], function () {
    // Define VS Code Dark+ theme
    monaco.editor.setTheme('vs-dark');

    editor = monaco.editor.create(document.getElementById('monaco-container'), {
      value: '// Loading file...',
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
      editor.setValue(`// File not found or unreadable: ${relPath}`);
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
    editor.setValue(`// Error loading file: ${err.message}`);
  }
}
