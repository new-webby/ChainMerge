const DEFAULT_API_URL = 'http://localhost:8080';

// ── Regex Helpers ─────────────────────────────────────────────
const REGEX = {
  ethereum: /^0x[a-fA-F0-9]{64}$/,
  solana:   /^[1-9A-HJ-NP-Za-km-z]{32,88}$/,
  bitcoin:  /^[a-fA-F0-9]{64}$/, // Simplified for TXID
  cosmos:   /^[A-F0-9]{64}$/,
  aptos:    /^0x[a-fA-F0-9]{60,66}$/, // Typically 32 bytes hex
  sui:      /^[1-9A-HJ-NP-Za-km-z]{43,45}$/,
  starknet: /^0x[a-fA-F0-9]{63,66}$/,
  polkadot: /^0x[a-fA-F0-9]{64}$/
};

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

const cmKeyStatus     = document.getElementById('cm-key-status');

const apiUrlInput   = document.getElementById('api-url-input');
const saveUrlBtn    = document.getElementById('save-url-btn');
const apiUrlDisplay = document.getElementById('api-url-display');
const healthDot     = document.getElementById('health-dot');
const toast         = document.getElementById('toast');
const alertsContainer = document.getElementById('alerts-container');

const WHALE_ETH_THRESHOLD = 50; 
const WHALE_TOKEN_THRESHOLD = 100000;

let state = {
  chainmergeApiKey: '',
  apiUrl: DEFAULT_API_URL,
  isDecoding: false
};

// ── Initialization ─────────────────────────────────────────────
chrome.storage.local.get(['chainmergeApiKey', 'chainmergeApiUrl'], (res) => {
  if (res.chainmergeApiKey) {
    state.chainmergeApiKey = res.chainmergeApiKey;
    cmKeyInput.value = res.chainmergeApiKey;
    setKeyStatus(cmKeyStatus, true);
  }
  state.apiUrl = res.chainmergeApiUrl || DEFAULT_API_URL;
  apiUrlInput.value = state.apiUrl;
  updateApiDisplay(state.apiUrl);
  checkHealth(state.apiUrl);
  
  initPopup();
});

async function initPopup() {
  const detected = await detectAndFill();
  if (detected) {
    // If we automatically filled a hash, auto-decode it
    handleDecode();
  }
}

// ── Detection Logic ───────────────────────────────────────────
async function detectAndFill() {
  // 1. Try Tab Context first
  const tabDetected = await detectFromTab();
  if (tabDetected) {
    fillForm(tabDetected.chain, tabDetected.hash);
    showToast(`Detected ${tabDetected.chain} tx from page`);
    return true;
  }

  // 2. Try Clipboard
  const clipDetected = await detectFromClipboard();
  if (clipDetected) {
    fillForm(clipDetected.chain, clipDetected.hash);
    showToast(`Found ${clipDetected.chain} hash in clipboard`);
    return true;
  }

  return false;
}

async function detectFromTab() {
  const [tab] = await chrome.tabs.query({ active: true, currentWindow: true });
  if (!tab || !tab.url) return null;

  try {
    const url = new URL(tab.url);
    const host = url.hostname;
    const path = url.pathname;

    if (host.includes('etherscan.io') || host.includes('bscscan.com') || host.includes('polygonscan.com') || host.includes('arbiscan.io')) {
      const m = path.match(/\/tx\/(0x[a-fA-F0-9]{64})/);
      if (m) return { chain: 'ethereum', hash: m[1] };
    } else if (host.includes('solscan.io') || host.includes('solana.fm')) {
      const m = path.match(/\/tx\/([1-9A-HJ-NP-Za-km-z]{32,88})/);
      if (m) return { chain: 'solana', hash: m[1] };
    } else if (host.includes('starkscan.co')) {
      const m = path.match(/\/tx\/(0x[a-fA-F0-9]+)/);
      if (m) return { chain: 'starknet', hash: m[1] };
    } else if (host.includes('mintscan.io')) {
      const m = path.match(/\/txs\/([A-F0-9]{64})/i);
      if (m) return { chain: 'cosmos', hash: m[1] };
    } else if (host.includes('aptoscan.com')) {
      const m = path.match(/\/tx\/(0x[a-fA-F0-9]+)/);
      if (m) return { chain: 'aptos', hash: m[1] };
    } else if (host.includes('suiscan.xyz')) {
      const m = path.match(/\/tx\/([1-9A-HJ-NP-Za-km-z]+)/);
      if (m) return { chain: 'sui', hash: m[1] };
    }
  } catch (e) {}
  return null;
}

async function detectFromClipboard() {
  try {
    const text = await navigator.clipboard.readText();
    const val = text.trim();
    if (!val) return null;

    // Simple priority matching
    if (REGEX.ethereum.test(val)) return { chain: 'ethereum', hash: val };
    if (REGEX.solana.test(val))   return { chain: 'solana', hash: val };
    if (REGEX.cosmos.test(val))   return { chain: 'cosmos', hash: val };
    if (REGEX.bitcoin.test(val))  return { chain: 'bitcoin', hash: val };
    if (REGEX.aptos.test(val))    return { chain: 'aptos', hash: val };
    if (REGEX.sui.test(val))      return { chain: 'sui', hash: val };
    if (REGEX.starknet.test(val)) return { chain: 'starknet', hash: val };
  } catch (err) {
    console.warn('Clipboard read failed:', err);
  }
  return null;
}

function fillForm(chain, hash) {
  chainSelect.value = chain;
  hashInput.value = hash;
  clearHashBtn.style.display = 'flex';
}

// ── Event Listeners ──────────────────────────────────────────
hashInput.addEventListener('input', () => {
  const val = hashInput.value.trim();
  clearHashBtn.style.display = val ? 'flex' : 'none';
  if (!val) return;

  // Manual input detection
  if (REGEX.ethereum.test(val)) chainSelect.value = 'ethereum';
  else if (REGEX.solana.test(val)) chainSelect.value = 'solana';
  else if (REGEX.bitcoin.test(val)) chainSelect.value = 'bitcoin';
});

clearHashBtn.addEventListener('click', () => {
  hashInput.value = '';
  clearHashBtn.style.display = 'none';
  hashInput.focus();
});

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

decodeBtn.addEventListener('click', handleDecode);

// ── Decode Logic ───────────────────────────────────────────────
async function handleDecode() {
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
      renderAlerts(result.decoded);
      renderOutput(result.decoded);
      explainWithAi(result.decoded);
    } else {
      renderError(result?.error || 'Decoding failed. Check if API is running.');
    }
  } catch (err) {
    renderError(err.message);
  } finally {
    setLoading(false);
  }
}

function renderAlerts(tx) {
  alertsContainer.textContent = '';
  const event = tx.events?.[0] || {};
  const alerts = [];

  // Whale detection
  const val = parseFloat(tx.value || '0');
  const nativeVal = val > 1e15 ? val / 1e18 : val;
  if (nativeVal > WHALE_ETH_THRESHOLD) {
    alerts.push({ type: 'whale', msg: `🐋 Whale alert — large value transfer detected` });
  }

  // Token whale
  const amount = parseFloat(event.amount || '0');
  const tokenAmount = amount > 1e15 ? amount / 1e18 : amount;
  if (tokenAmount > WHALE_TOKEN_THRESHOLD) {
    alerts.push({ type: 'whale', msg: `🐋 Whale alert — ${tokenAmount.toLocaleString()} tokens moved` });
  }

  // Risk: zero-value send
  if (tx.value === '0' && event.event_type !== 'token_transfer') {
    alerts.push({ type: 'risk', msg: `⚠ Zero-value contract interaction` });
  }

  alerts.forEach(alert => {
    const el = document.createElement('div');
    el.className = `cm-alert cm-alert-${alert.type}`;
    el.style.marginBottom = '10px';
    el.textContent = alert.msg;
    alertsContainer.appendChild(el);
  });
}

function renderOutput(data) {
  resultsArea.style.display = 'block';
  resultsArea.classList.remove('animate-in');
  void resultsArea.offsetWidth;
  resultsArea.classList.add('animate-in');
  
  outputPre.textContent = '';
  const highlightedNodes = syntaxHighlightToNodes(JSON.stringify(data, null, 2));
  highlightedNodes.forEach(node => outputPre.appendChild(node));
}

function renderError(msg) {
  resultsArea.style.display = 'block';
  resultsArea.classList.remove('animate-in');
  void resultsArea.offsetWidth;
  resultsArea.classList.add('animate-in');
  
  outputPre.textContent = '';
  const errorSpan = document.createElement('span');
  errorSpan.style.color = 'var(--red)';
  errorSpan.textContent = msg;
  outputPre.appendChild(errorSpan);
}

function setLoading(isLoading) {
  state.isDecoding = isLoading;
  decodeBtn.disabled = isLoading;
  decodeBtn.textContent = '';
  
  if (isLoading) {
    const spinner = document.createElement('div');
    spinner.className = 'spinner';
    decodeBtn.appendChild(spinner);
    decodeBtn.appendChild(document.createTextNode(' Decoding...'));
  } else {
    const icon = document.createElement('span');
    icon.textContent = '⚡';
    decodeBtn.appendChild(icon);
    decodeBtn.appendChild(document.createTextNode(' Decode Transaction'));
  }
}

async function explainWithAi(decoded) {
  aiBox.style.display = 'block';
  aiBox.classList.remove('animate-in');
  void aiBox.offsetWidth;
  aiBox.classList.add('animate-in');
  
  aiContent.textContent = '';
  const spinner = document.createElement('div');
  spinner.className = 'spinner';
  spinner.style.margin = '8px 0';
  aiContent.appendChild(spinner);
  aiContent.appendChild(document.createTextNode(' Generating explanation...'));

  chrome.runtime.sendMessage({ 
    type: 'CM_GEMINI_EXPLAIN', 
    decoded 
  }, (res) => {
    aiContent.textContent = '';
    if (res && res.ok) {
      aiContent.textContent = res.text;
    } else {
      const errorSpan = document.createElement('span');
      errorSpan.style.color = 'var(--red)';
      errorSpan.style.fontSize = '10px';
      errorSpan.textContent = `AI Error: ${res?.error || 'Failed to generate explanation'}`;
      aiContent.appendChild(errorSpan);
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

function syntaxHighlightToNodes(json) {
  const nodes = [];
  const regex = /("(\\u[a-zA-Z0-9]{4}|\\[^u]|[^\\"])*"(\s*:)?|\b(true|false|null)\b|-?\d+\.?\d*([eE][+-]?\d+)?)/g;
  let lastIndex = 0;
  let match;

  while ((match = regex.exec(json)) !== null) {
    // Text before the match
    if (match.index > lastIndex) {
      nodes.push(document.createTextNode(json.substring(lastIndex, match.index)));
    }

    const value = match[0];
    let cls = 'json-num';
    if (/^"/.test(value)) {
      if (/:$/.test(value)) cls = 'json-key';
      else cls = 'json-str';
    } else if (/true|false/.test(value)) {
      cls = 'json-bool';
    } else if (/null/.test(value)) {
      cls = 'json-null';
    }

    const span = document.createElement('span');
    span.className = cls;
    span.textContent = value;
    nodes.push(span);

    lastIndex = regex.lastIndex;
  }

  // Remaining text
  if (lastIndex < json.length) {
    nodes.push(document.createTextNode(json.substring(lastIndex)));
  }

  return nodes;
}
