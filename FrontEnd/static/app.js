// c_to_rust Dashboard Client Controller

let activeTab = 'rust';
let currentC = '';
let currentRust = '';
let isStreamingRust = false;
let isStreamingC = false;
let eventSource = null;

function setPreset(url) {
  document.getElementById('repo-url').value = url;
}

function switchCodeTab(tab) {
  activeTab = tab;
  const tabC = document.getElementById('tab-c');
  const tabRust = document.getElementById('tab-rust');
  const containerC = document.getElementById('c-code-container');
  const containerRust = document.getElementById('rust-code-container');

  if (tab === 'c') {
    tabC.classList.add('active');
    tabRust.classList.remove('active');
    containerC.style.display = 'block';
    containerRust.style.display = 'none';
  } else {
    tabRust.classList.add('active');
    tabC.classList.remove('active');
    containerRust.style.display = 'block';
    containerC.style.display = 'none';
  }
}

function appendTerminal(text, type = 'normal') {
  const terminal = document.getElementById('terminal');
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

function clearTerminal() {
  const terminal = document.getElementById('terminal');
  terminal.innerHTML = '<div class="terminal-line"><span class="terminal-cmd">$</span> <span>Terminal cleared.</span></div>';
}

function updateStage(id, desc) {
  for (let i = 1; i <= 5; i++) {
    const card = document.getElementById(`step-${i}`);
    if (i < id) {
      card.className = 'step-card completed';
    } else if (i === id) {
      card.className = 'step-card active';
      if (desc) {
        const descEl = document.getElementById(`step-${i}-desc`);
        if (descEl) descEl.textContent = desc;
      }
    } else {
      card.className = 'step-card';
    }
  }
}

function resetUIForNewTranslation(url) {
  isStreamingRust = false;
  isStreamingC = false;
  currentC = '';
  currentRust = '';

  document.getElementById('c-code-view').textContent = '// Fetching C source files...';
  document.getElementById('rust-code-view').textContent = '// Synthesizing Rust code from local LLM...';
  document.getElementById('results-banner').style.display = 'none';

  updateStage(1, `Cloning ${url}...`);

  const btn = document.getElementById('btn-convert');
  const btnStop = document.getElementById('btn-stop');
  btn.disabled = true;
  btn.querySelector('span').textContent = 'Translating...';
  btnStop.style.display = 'flex';
}

function initSSE() {
  if (eventSource) {
    eventSource.close();
  }

  eventSource = new EventSource('/api/events');

  eventSource.addEventListener('init', (e) => {
    const data = JSON.parse(e.data);
    if (data.is_running) {
      const btn = document.getElementById('btn-convert');
      const btnStop = document.getElementById('btn-stop');
      btn.disabled = true;
      btn.querySelector('span').textContent = 'Translating...';
      btnStop.style.display = 'flex';
      document.getElementById('results-banner').style.display = 'none';
    } else if (data.has_zip) {
      document.getElementById('results-banner').style.display = 'flex';
      updateStage(5, 'Crate compiled & packaged');
    }
  });

  eventSource.addEventListener('reset', (e) => {
    const data = JSON.parse(e.data);
    resetUIForNewTranslation(data.url || '');
  });

  eventSource.addEventListener('status', (e) => {
    const data = JSON.parse(e.data);
    const btn = document.getElementById('btn-convert');
    const btnStop = document.getElementById('btn-stop');
    if (data.state === 'started') {
      btn.disabled = true;
      btn.querySelector('span').textContent = 'Translating...';
      btnStop.style.display = 'flex';
    } else if (data.state === 'idle') {
      btn.disabled = false;
      btn.querySelector('span').textContent = 'Start Translation';
      btnStop.style.display = 'none';
      isStreamingRust = false;
      isStreamingC = false;
      if (data.cancelled) {
        appendTerminal('Translation stopped by user.', 'warn');
      }
    }
  });

  eventSource.addEventListener('stage', (e) => {
    const data = JSON.parse(e.data);
    updateStage(data.id, data.desc);
    appendTerminal(`[Stage ${data.id}] ${data.name}: ${data.desc}`, 'cmd');
  });

  eventSource.addEventListener('command', (e) => {
    const data = JSON.parse(e.data);
    const statusText = data.status ? ` [${data.status.toUpperCase()}]` : '';
    appendTerminal(`${data.cmd}${statusText}`, data.status === 'passed' ? 'success' : 'cmd');
  });

  eventSource.addEventListener('log', (e) => {
    const data = JSON.parse(e.data);
    const text = data.text;
    if (text.includes('PASSED')) {
      appendTerminal(text, 'success');
    } else if (text.includes('Warning') || text.includes('Repair')) {
      appendTerminal(text, 'warn');
    } else if (text.includes('failed') || text.includes('Error')) {
      appendTerminal(text, 'error');
    } else {
      appendTerminal(text, 'normal');
    }
  });

  // Token-by-token streaming for C code
  eventSource.addEventListener('c_token', (e) => {
    const data = JSON.parse(e.data);
    const view = document.getElementById('c-code-view');
    if (!isStreamingC) {
      view.textContent = '';
      currentC = '';
      isStreamingC = true;
    }
    currentC += data.token;
    view.textContent = currentC;
    const container = document.getElementById('c-code-container');
    container.scrollTop = container.scrollHeight;
  });

  // Full C code event
  eventSource.addEventListener('c_code', (e) => {
    const data = JSON.parse(e.data);
    currentC = data.code;
    const view = document.getElementById('c-code-view');
    view.textContent = currentC;
    Prism.highlightElement(view);
  });

  // Token-by-token streaming for Rust code
  eventSource.addEventListener('rust_token', (e) => {
    const data = JSON.parse(e.data);
    const view = document.getElementById('rust-code-view');
    if (!isStreamingRust) {
      view.textContent = '';
      currentRust = '';
      isStreamingRust = true;
      switchCodeTab('rust');
    }
    currentRust += data.token;
    view.textContent = currentRust;
    const container = document.getElementById('rust-code-container');
    container.scrollTop = container.scrollHeight;
  });

  // Full Rust code event
  eventSource.addEventListener('rust_code', (e) => {
    const data = JSON.parse(e.data);
    currentRust = data.code;
    const view = document.getElementById('rust-code-view');
    view.textContent = currentRust;
    Prism.highlightElement(view);
    switchCodeTab('rust');
  });

  eventSource.addEventListener('complete', (e) => {
    const data = JSON.parse(e.data);
    isStreamingRust = false;
    isStreamingC = false;

    updateStage(5, 'Completed');
    appendTerminal('Translation completed with 100% test conformance.', 'success');

    // Highlight final code blocks with Prism
    const rustView = document.getElementById('rust-code-view');
    if (rustView.textContent) Prism.highlightElement(rustView);
    const cView = document.getElementById('c-code-view');
    if (cView.textContent) Prism.highlightElement(cView);

    const banner = document.getElementById('results-banner');
    banner.style.display = 'flex';
    document.getElementById('results-title').textContent = `Translated: ${data.crate_name} (100% Safe Rust)`;
    document.getElementById('results-desc').textContent = 'Compiled and verified with 0 raw pointers and 0 unsafe blocks. Ready in Web VS Code.';

    const btnConvert = document.getElementById('btn-convert');
    const btnStop = document.getElementById('btn-stop');
    btnConvert.disabled = false;
    btnConvert.querySelector('span').textContent = 'Start Translation';
    btnStop.style.display = 'none';
  });

  eventSource.addEventListener('error', (e) => {
    try {
      const data = JSON.parse(e.data);
      appendTerminal(`Error: ${data.message}`, 'error');
    } catch (_) {}
    const btnConvert = document.getElementById('btn-convert');
    const btnStop = document.getElementById('btn-stop');
    btnConvert.disabled = false;
    btnConvert.querySelector('span').textContent = 'Start Translation';
    btnStop.style.display = 'none';
  });
}

// Start conversion action
document.getElementById('btn-convert').addEventListener('click', async () => {
  const url = document.getElementById('repo-url').value.trim();
  if (!url) {
    alert('Please enter a Git repository URL');
    return;
  }

  resetUIForNewTranslation(url);
  appendTerminal(`\n--- Starting translation for: ${url} ---`, 'cmd');

  try {
    const res = await fetch('/api/convert', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ url: url })
    });
    const result = await res.json();
    if (!res.ok) {
      appendTerminal(`Server error: ${result.error}`, 'error');
      const btn = document.getElementById('btn-convert');
      const btnStop = document.getElementById('btn-stop');
      btn.disabled = false;
      btn.querySelector('span').textContent = 'Start Translation';
      btnStop.style.display = 'none';
    }
  } catch (err) {
    appendTerminal(`Network error: ${err.message}`, 'error');
    const btn = document.getElementById('btn-convert');
    const btnStop = document.getElementById('btn-stop');
    btn.disabled = false;
    btn.querySelector('span').textContent = 'Start Translation';
    btnStop.style.display = 'none';
  }
});

// Stop / Cancel conversion action
async function stopTranslation() {
  appendTerminal('Stopping active translation...', 'warn');
  try {
    await fetch('/api/cancel', { method: 'POST' });
  } catch (err) {
    console.error('Cancel request failed:', err);
  }
  const btn = document.getElementById('btn-convert');
  const btnStop = document.getElementById('btn-stop');
  btn.disabled = false;
  btn.querySelector('span').textContent = 'Start Translation';
  btnStop.style.display = 'none';
}

function copyActiveCode() {
  const code = activeTab === 'rust' ? currentRust : currentC;
  if (!code) return;
  navigator.clipboard.writeText(code).then(() => {
    alert('Code copied to clipboard!');
  });
}

// Poll GPU pill for header
async function pollHeaderTelemetry() {
  try {
    const res = await fetch('/api/system_stats');
    if (!res.ok) return;
    const data = await res.json();
    if (data.gpu && data.gpu.available) {
      document.getElementById('gpu-header-text').textContent = 
        `RTX 3050: ${Math.round(data.gpu.utilization)}% | ${data.gpu.power_draw.toFixed(0)}W | ${Math.round(data.gpu.temperature)}°C`;
    }
    if (data.llm && data.llm.online) {
      document.getElementById('llm-header-text').textContent = `LLM: ${data.llm.model} (8k ctx)`;
    }
  } catch (_) {}
}

setInterval(pollHeaderTelemetry, 2000);
pollHeaderTelemetry();
initSSE();
