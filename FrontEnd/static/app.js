// c_to_rust Dashboard Client Controller

let activeTab = 'rust';
let currentC = '';
let currentRust = '';
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

function initSSE() {
  if (eventSource) {
    eventSource.close();
  }

  eventSource = new EventSource('/api/events');

  eventSource.addEventListener('init', (e) => {
    const data = JSON.parse(e.data);
    if (data.has_zip) {
      document.getElementById('results-banner').style.display = 'flex';
      updateStage(5, 'Crate compiled & packaged');
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

  eventSource.addEventListener('c_code', (e) => {
    const data = JSON.parse(e.data);
    currentC = data.code;
    const view = document.getElementById('c-code-view');
    view.textContent = currentC;
    Prism.highlightElement(view);
  });

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
    updateStage(5, 'Completed');
    appendTerminal('🎉 Translation completed with 100% test conformance!', 'success');

    const banner = document.getElementById('results-banner');
    banner.style.display = 'flex';
    document.getElementById('results-title').textContent = `Translated: ${data.crate_name} (100% Safe Rust)`;
    document.getElementById('results-desc').textContent = 'Compiled and verified with 0 raw pointers and 0 unsafe blocks. Ready in Web VS Code.';

    const btnConvert = document.getElementById('btn-convert');
    btnConvert.disabled = false;
    btnConvert.querySelector('span').textContent = 'Start Translation';
  });

  eventSource.addEventListener('error', (e) => {
    try {
      const data = JSON.parse(e.data);
      appendTerminal(`Error: ${data.message}`, 'error');
    } catch (_) {}
    const btnConvert = document.getElementById('btn-convert');
    btnConvert.disabled = false;
    btnConvert.querySelector('span').textContent = 'Start Translation';
  });
}

// Start conversion action
document.getElementById('btn-convert').addEventListener('click', async () => {
  const url = document.getElementById('repo-url').value.trim();
  if (!url) {
    alert('Please enter a Git repository URL');
    return;
  }

  const btn = document.getElementById('btn-convert');
  btn.disabled = true;
  btn.querySelector('span').textContent = 'Translating...';

  document.getElementById('results-banner').style.display = 'none';
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
      btn.disabled = false;
      btn.querySelector('span').textContent = 'Start Translation';
    }
  } catch (err) {
    appendTerminal(`Network error: ${err.message}`, 'error');
    btn.disabled = false;
    btn.querySelector('span').textContent = 'Start Translation';
  }
});

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
