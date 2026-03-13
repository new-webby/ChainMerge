const DEFAULT_API_URL = 'http://localhost:8080';

// ── Elements ──────────────────────────────────────────────────
const tabBtns     = document.querySelectorAll('.tab-btn');
const tabContents = document.querySelectorAll('.tab-content');

const chainSelect  = document.getElementById('chain-select');
const hashInput    = document.getElementById('hash-input');
const clearHashBtn = document.getElementById('clear-hash-btn');
const decodeBtn    = document.getElementById('decode-btn');
const resultsArea  = document.getElementById('results-area');
const outputPre    = document.getElementById('output-pre');
const copyBtn      = document.getElementById('copy-btn');
const aiBox        = document.getElementById('ai-box');
const aiContent    = document.getElementById('ai-content');

const keyInput      = document.getElementById('gemini-key-input');
const saveKeyBtn    = document.getElementById('save-key-btn');
const clearKeyBtn   = document.getElementById('clear-key-btn');
const keyStatus     = document.getElementById('key-status');

const cmKeyInput      = document.getElementById('cm-key-input');
const saveCmKeyBtn    = document.getElementById('save-cm-key-btn');
const clearCmKeyBtn   = document.getElementById('clear-cm-key-btn');
const cmKeyStatus     = document.getElementById('cm-key-status');

const apiUrlInput   = document.getElementById('api-url-input');
const saveUrlBtn    = document.getElementById('save-url-btn');
const apiUrlDisplay = document.getElementById('api-url-display');
const healthDot     = document.getElementById('health-dot');
const toast         = document.getElementById('toast');

let state = {
  geminiApiKey: '',
  chainmergeApiKey: '',
  apiUrl: DEFAULT_API_URL,
  isDecoding: false
};

// ── Initialization ─────────────────────────────────────────────
chrome.storage.local.get(['geminiApiKey', 'chainmergeApiKey', 'chainmergeApiUrl'], (res) => {
  if (res.geminiApiKey) {
    state.geminiApiKey = res.geminiApiKey;
    keyInput.value = res.geminiApiKey;
    setKeyStatus(keyStatus, true);
  }
  if (res.chainmergeApiKey) {
    state.chainmergeApiKey = res.chainmergeApiKey;
    cmKeyInput.value = res.chainmergeApiKey;
    setKeyStatus(cmKeyStatus, true);
  }
  state.apiUrl = res.chainmergeApiUrl || DEFAULT_API_URL;
  apiUrlInput.value = state.apiUrl;
  updateApiDisplay(state.apiUrl);
  checkHealth(state.apiUrl);
  
  detectTabContext();
});

// ── Tab Context Detection ──────────────────────────────────────
async function detectTabContext() {
  const [tab] = await chrome.tabs.query({ active: true, currentWindow: true });
  if (!tab || !tab.url) return;

  try {
    const url = new URL(tab.url);
    const host = url.hostname;
    const path = url.pathname;

    let detected = null;

    if (host.includes('etherscan.io') || host.includes('bscscan.com') || host.includes('polygonscan.com') || host.includes('arbiscan.io')) {
      const m = path.match(/\/tx\/(0x[a-fA-F0-9]{64})/);
      if (m) detected = { chain: 'ethereum', hash: m[1] };
    } else if (host.includes('solscan.io') || host.includes('solana.fm')) {
      const m = path.match(/\/tx\/([1-9A-HJ-NP-Za-km-z]{32,88})/);
      if (m) detected = { chain: 'solana', hash: m[1] };
    } else if (host.includes('starkscan.co')) {
      const m = path.match(/\/tx\/(0x[a-fA-F0-9]+)/);
      if (m) detected = { chain: 'starknet', hash: m[1] };
    } else if (host.includes('mintscan.io')) {
      const m = path.match(/\/txs\/([A-F0-9]{64})/i);
      if (m) detected = { chain: 'cosmos', hash: m[1] };
    } else if (host.includes('aptoscan.com')) {
      const m = path.match(/\/tx\/(0x[a-fA-F0-9]+)/);
      if (m) detected = { chain: 'aptos', hash: m[1] };
    } else if (host.includes('suiscan.xyz')) {
      const m = path.match(/\/tx\/([1-9A-HJ-NP-Za-km-z]+)/);
      if (m) detected = { chain: 'sui', hash: m[1] };
    }

    if (detected) {
      chainSelect.value = detected.chain;
      hashInput.value = detected.hash;
      clearHashBtn.style.display = 'flex';
      showToast(`Detected ${detected.chain} transaction`);
    }
  } catch (e) {}
}

// ── Auto-Detection Logic ───────────────────────────────────────
hashInput.addEventListener('input', () => {
  const val = hashInput.value.trim();
  clearHashBtn.style.display = val ? 'flex' : 'none';
  if (!val) return;

  if (val.startsWith('0x') && val.length === 66) {
    chainSelect.value = 'ethereum';
  } 
  else if (!val.startsWith('0x') && val.length >= 32 && val.length <= 88 && /^[1-9A-HJ-NP-Za-km-z]+$/.test(val)) {
    chainSelect.value = 'solana';
  }
  else if (!val.startsWith('0x') && val.length === 64 && /^[0-9a-fA-F]+$/.test(val)) {
    chainSelect.value = 'bitcoin';
  }
});

clearHashBtn.addEventListener('click', () => {
  hashInput.value = '';
  clearHashBtn.style.display = 'none';
  hashInput.focus();
});

// ── Tab Switching ──────────────────────────────────────────────
tabBtns.forEach(btn => {
  btn.addEventListener('click', () => {
    const tab = btn.dataset.tab;
    if (!tab) return;
    
    tabBtns.forEach(b => b.classList.remove('active'));
    tabContents.forEach(c => c.classList.remove('active'));
    
    btn.classList.add('active');
    document.getElementById(`${tab}-tab`).classList.add('active');
  });
});

// ── Decode Logic ───────────────────────────────────────────────
decodeBtn.addEventListener('click', async () => {
  const chain = chainSelect.value;
  const hash = hashInput.value.trim();

  if (!hash) {
    showToast('Please enter a transaction hash');
    hashInput.focus();
    return;
  }

  setLoading(true);
  resultsArea.style.display = 'none';
  aiBox.style.display = 'none';

  try {
    const result = await new Promise((resolve) => {
      chrome.runtime.sendMessage(
        { type: 'CM_DECODE', chain, hash, apiUrl: state.apiUrl, apiKey: state.chainmergeApiKey },
        resolve
      );
    });

    if (result && result.ok) {
      renderOutput(result.decoded);
      if (state.geminiApiKey) {
        explainWithAi(result.decoded);
      }
    } else {
      renderError(result?.error || 'Decoding failed. Check if API is running.');
    }
  } catch (err) {
    renderError(err.message);
  } finally {
    setLoading(false);
  }
});

function renderOutput(data) {
  resultsArea.style.display = 'block';
  outputPre.innerHTML = syntaxHighlight(JSON.stringify(data, null, 2));
}

function renderError(msg) {
  resultsArea.style.display = 'block';
  outputPre.innerHTML = `<span style="color:var(--red)">${msg}</span>`;
}

function setLoading(isLoading) {
  state.isDecoding = isLoading;
  decodeBtn.disabled = isLoading;
  decodeBtn.innerHTML = isLoading 
    ? `<div class="spinner"></div> Decoding...` 
    : `<span>⚡</span> Decode Transaction`;
}

async function explainWithAi(decoded) {
  aiBox.style.display = 'block';
  aiContent.innerHTML = `<div class="spinner" style="margin: 8px 0;"></div> Generating explanation...`;

  chrome.runtime.sendMessage({ 
    type: 'CM_GEMINI_EXPLAIN', 
    apiKey: state.geminiApiKey,
    decoded 
  }, (res) => {
    if (res && res.ok) {
      aiContent.textContent = res.text;
    } else {
      aiContent.innerHTML = `<span style="color:var(--red); font-size:10px;">AI Error: ${res?.error || 'Failed to generate explanation'}</span>`;
    }
  });
}

// ── Settings Logic ─────────────────────────────────────────────
saveCmKeyBtn.addEventListener('click', () => {
  const key = cmKeyInput.value.trim();
  if (!key) return;
  chrome.storage.local.set({ chainmergeApiKey: key }, () => {
    state.chainmergeApiKey = key;
    setKeyStatus(cmKeyStatus, true);
    showToast('ChainMerge key saved!');
    checkHealth(state.apiUrl);
  });
});

clearCmKeyBtn.addEventListener('click', () => {
  chrome.storage.local.remove('chainmergeApiKey', () => {
    state.chainmergeApiKey = '';
    cmKeyInput.value = '';
    setKeyStatus(cmKeyStatus, false);
    showToast('ChainMerge key cleared');
    checkHealth(state.apiUrl);
  });
});

saveKeyBtn.addEventListener('click', () => {
  const key = keyInput.value.trim();
  if (!key) return;
  chrome.storage.local.set({ geminiApiKey: key }, () => {
    state.geminiApiKey = key;
    setKeyStatus(keyStatus, true);
    showToast('Gemini key saved!');
  });
});

clearKeyBtn.addEventListener('click', () => {
  chrome.storage.local.remove('geminiApiKey', () => {
    state.geminiApiKey = '';
    keyInput.value = '';
    setKeyStatus(keyStatus, false);
    showToast('Gemini key cleared');
  });
});

saveUrlBtn.addEventListener('click', () => {
  let url = apiUrlInput.value.trim().replace(/\/+$/, '');
  if (!url) url = DEFAULT_API_URL;
  chrome.storage.local.set({ chainmergeApiUrl: url }, () => {
    state.apiUrl = url;
    updateApiDisplay(url);
    checkHealth(url);
    showToast('URL updated!');
  });
});

copyBtn.addEventListener('click', () => {
  const text = outputPre.textContent;
  navigator.clipboard.writeText(text).then(() => {
    showToast('Copied to clipboard!');
  });
});

// ── Helpers ────────────────────────────────────────────────────
function setKeyStatus(el, isSet) {
  el.textContent = isSet ? 'Saved' : 'Not Set';
  el.className = `status-pill ${isSet ? 'set' : 'unset'}`;
}

function updateApiDisplay(url) {
  try {
    const parsed = new URL(url);
    apiUrlDisplay.textContent = parsed.host;
  } catch { apiUrlDisplay.textContent = url; }
}

async function checkHealth(url) {
  healthDot.className = 'health-dot';
  chrome.runtime.sendMessage({ 
    type: 'CM_HEALTH', 
    apiUrl: url, 
    apiKey: state.chainmergeApiKey 
  }, (res) => {
    healthDot.className = `health-dot ${res && res.ok ? 'online' : 'offline'}`;
  });
}

function showToast(msg) {
  toast.textContent = msg;
  toast.classList.add('visible');
  setTimeout(() => toast.classList.remove('visible'), 2500);
}

function syntaxHighlight(json) {
  return json.replace(/("(\\u[a-zA-Z0-9]{4}|\\[^u]|[^\\"])*"(\s*:)?|\b(true|false|null)\b|-?\d+\.?\d*([eE][+-]?\d+)?)/g, (match) => {
    let cls = 'json-num';
    if (/^"/.test(match)) {
      if (/:$/.test(match)) cls = 'json-key';
      else cls = 'json-str';
    }
    return `<span class="${cls}">${match}</span>`;
  });
}
