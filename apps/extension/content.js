/* content.js — ChainMerge TX Decoder Content Script */
/* Injected into block explorer pages to detect and decode transactions */

/* Firefox/Chrome compatibility shim */
const _chr = (typeof browser !== 'undefined' && browser.runtime) ? browser : chrome;

(function () {
  'use strict';

  // ── Constants ────────────────────────────────────────────────
  const DEFAULT_API_URL  = 'http://localhost:8080';
  const GEMINI_API_BASE  = 'https://generativelanguage.googleapis.com/v1beta/models/gemini-2.5-flash:generateContent';
  const WHALE_ETH_THRESHOLD = 50; // ETH
  const WHALE_TOKEN_THRESHOLD = 100000; // generic token units

  const CHAIN_EMOJI = {
    ethereum: '⟠', solana: '◎', cosmos: '⚛', aptos: '▲',
    sui: '💧', polkadot: '●', bitcoin: '₿', starknet: '★',
  };

  // ── Explorer Detection ────────────────────────────────────────
  function detectExplorerInfo() {
    const host = location.hostname;
    const path = location.pathname;

    // Ethereum-family (EVM)
    if (host.includes('etherscan.io')) {
      const m = path.match(/\/tx\/(0x[a-fA-F0-9]{64})/);
      return m ? { chain: 'ethereum', hash: m[1] } : null;
    }
    if (host.includes('bscscan.com') || host.includes('polygonscan.com') || host.includes('arbiscan.io')) {
      const m = path.match(/\/tx\/(0x[a-fA-F0-9]{64})/);
      return m ? { chain: 'ethereum', hash: m[1] } : null;
    }
    // Solana
    if (host.includes('solscan.io')) {
      const m = path.match(/\/tx\/([1-9A-HJ-NP-Za-km-z]{43,88})/);
      return m ? { chain: 'solana', hash: m[1] } : null;
    }
    if (host.includes('solana.fm')) {
      const m = path.match(/\/tx\/([1-9A-HJ-NP-Za-km-z]{43,88})/);
      return m ? { chain: 'solana', hash: m[1] } : null;
    }
    // StarkNet
    if (host.includes('starkscan.co')) {
      const m = path.match(/\/tx\/(0x[a-fA-F0-9]+)/);
      return m ? { chain: 'starknet', hash: m[1] } : null;
    }
    // Cosmos / Mintscan
    if (host.includes('mintscan.io')) {
      const m = path.match(/\/txs\/([A-F0-9]{64})/i);
      return m ? { chain: 'cosmos', hash: m[1].toUpperCase() } : null;
    }
    // Aptos
    if (host.includes('aptoscan.com')) {
      const m = path.match(/\/tx\/(0x[a-fA-F0-9]+)/);
      return m ? { chain: 'aptos', hash: m[1] } : null;
    }
    // Sui
    if (host.includes('suiscan.xyz')) {
      const m = path.match(/\/tx\/([1-9A-HJ-NP-Za-km-z]+)/);
      return m ? { chain: 'sui', hash: m[1] } : null;
    }
    // Bitcoin
    if (host.includes('blockchain.com') || host.includes('blockstream.info')) {
      const m = path.match(/\/tx\/([a-fA-F0-9]{64})/);
      return m ? { chain: 'bitcoin', hash: m[1] } : null;
    }
    // Polkadot
    if (host.includes('subscan.io')) {
      const m = path.match(/\/extrinsic\/(0x[a-fA-F0-9]{64})/);
      return m ? { chain: 'polkadot', hash: m[1] } : null;
    }
    if (host.includes('polkascan.io')) {
      const m = path.match(/\/transaction\/(0x[a-fA-F0-9]{64})/);
      return m ? { chain: 'polkadot', hash: m[1] } : null;
    }

    return null;
  }

  const explorerInfo = detectExplorerInfo();
  if (!explorerInfo) return; // not a tx page

  // ── Build UI ──────────────────────────────────────────────────
  function truncate(str, n = 8) {
    if (!str) return '—';
    if (str.length <= n * 2 + 3) return str;
    return `${str.slice(0, n)}…${str.slice(-n)}`;
  }

  // Toggle button
  const toggle = document.createElement('button');
  toggle.className = 'cm-toggle';
  toggle.title = 'Toggle ChainMerge Panel';
  
  const toggleIcon = document.createElement('span');
  toggleIcon.className = 'cm-toggle-icon';
  toggleIcon.textContent = '⛓';
  toggle.appendChild(toggleIcon);
  toggle.appendChild(document.createTextNode(' CM'));
  document.body.appendChild(toggle);

  // Panel container
  const panel = document.createElement('div');
  panel.className = 'cm-panel';

  // Panel Header
  const header = document.createElement('div');
  header.className = 'cm-header';
  
  const headerLeft = document.createElement('div');
  headerLeft.className = 'cm-header-left';
  const logoIcon = document.createElement('span');
  logoIcon.style.fontSize = '18px';
  logoIcon.textContent = '⛓';
  const logoText = document.createElement('span');
  logoText.className = 'cm-logo-text';
  logoText.textContent = 'ChainMerge';
  headerLeft.appendChild(logoIcon);
  headerLeft.appendChild(logoText);

  const headerRight = document.createElement('div');
  headerRight.className = 'cm-header-right';
  const closeBtn = document.createElement('button');
  closeBtn.className = 'cm-close-btn';
  closeBtn.id = 'cm-close';
  closeBtn.textContent = '✕';
  headerRight.appendChild(closeBtn);

  header.appendChild(headerLeft);
  header.appendChild(headerRight);

  // Panel Body
  const panelBody = document.createElement('div');
  panelBody.className = 'cm-body';
  panelBody.id = 'cm-body';
  
  const loading = document.createElement('div');
  loading.className = 'cm-loading';
  const spinner = document.createElement('div');
  spinner.className = 'cm-spinner';
  const loadingText = document.createElement('span');
  loadingText.textContent = 'Decoding transaction…';
  loading.appendChild(spinner);
  loading.appendChild(loadingText);
  panelBody.appendChild(loading);

  // Panel Footer
  const footer = document.createElement('div');
  footer.className = 'cm-footer';
  const footerBrand = document.createElement('span');
  footerBrand.className = 'cm-footer-brand';
  footerBrand.textContent = 'Powered by ChainMerge SDK';
  const footerLink = document.createElement('a');
  footerLink.className = 'cm-footer-link';
  footerLink.href = 'http://localhost:5173';
  footerLink.target = '_blank';
  footerLink.textContent = 'Open Playground →';
  footer.appendChild(footerBrand);
  footer.appendChild(footerLink);

  panel.appendChild(header);
  panel.appendChild(panelBody);
  panel.appendChild(footer);

  document.body.appendChild(panel);

  // ── Toggle behaviour ─────────────────────────────────────────
  let isOpen = false;
  function openPanel()  { panel.classList.add('cm-open'); isOpen = true; }
  function closePanel() { panel.classList.remove('cm-open'); isOpen = false; }

  toggle.addEventListener('click', () => isOpen ? closePanel() : openPanel());
  panel.querySelector('#cm-close').addEventListener('click', closePanel);

  // Auto-open disabled per user request
  // setTimeout(openPanel, 700);

  // ── Decode + Render ─────────────────────────────────────────
  chrome.storage.local.get(['geminiApiKey', 'chainmergeApiKey', 'chainmergeApiUrl'], async ({ geminiApiKey, chainmergeApiKey, chainmergeApiUrl }) => {
    const apiUrl = (chainmergeApiUrl || DEFAULT_API_URL).replace(/\/+$/, '');
    const body   = document.getElementById('cm-body');

    let decoded = null;

    // Step 1: Decode via background service worker (bypasses PNA/CORS on localhost)
    const result = await new Promise((resolve) => {
      chrome.runtime.sendMessage(
        { type: 'CM_DECODE', chain: explorerInfo.chain, hash: explorerInfo.hash, apiUrl, apiKey: chainmergeApiKey },
        resolve
      );
    });

    if (!result || !result.ok) {
      renderError(body, result?.error || 'Could not connect to ChainMerge API', apiUrl);
      return;
    }

    decoded = result.decoded;

    // Step 2: Render decoded data
    renderDecoded(body, decoded, explorerInfo.chain);

    // Step 3: AI explanation (async, non-blocking)
    if (geminiApiKey) {
      renderAiLoading(body);
      try {
        const forensicData = await fetchGeminiExplanation(geminiApiKey, decoded);
        renderAiExplanation(body, forensicData);
      } catch (err) {
        renderAiError(body, err.message);
      }
    } else {
      renderAiNoKey(body);
    }
  });

  // ── Render: Decoded Data ─────────────────────────────────────
  function renderDecoded(body, tx, chain) {
    const emoji = CHAIN_EMOJI[chain] || '🔗';
    const event = tx.events?.[0] || {};
    const eventType = event.event_type || 'unknown';

    // Whale & Risk detection
    const alerts = detectAlerts(tx, event);

    body.textContent = '';

    // Chain row
    const chainRow = document.createElement('div');
    chainRow.className = 'cm-chain-row';
    const chainBadge = document.createElement('span');
    chainBadge.className = 'cm-chain-badge';
    chainBadge.textContent = `${emoji} ${chain}`;
    chainRow.appendChild(chainBadge);
    body.appendChild(chainRow);

    const hashEl = document.createElement('div');
    hashEl.className = 'cm-hash';
    hashEl.style.marginBottom = '12px';
    hashEl.textContent = truncate(tx.tx_hash, 12);
    hashEl.title = tx.tx_hash;
    body.appendChild(hashEl);

    // Alert banners
    alerts.forEach(({ type, msg }) => {
      const el = document.createElement('div');
      el.className = `cm-alert cm-alert-${type}`;
      el.textContent = msg;
      body.appendChild(el);
    });

    // Transaction card
    const card = document.createElement('div');
    card.className = 'cm-card';
    const cardTitle = document.createElement('div');
    cardTitle.className = 'cm-card-title';
    cardTitle.textContent = 'Transaction';
    card.appendChild(cardTitle);

    const typeBadge = document.createElement('span');
    typeBadge.className = `cm-event-badge cm-event-${eventType}`;
    typeBadge.textContent = formatEventType(eventType);
    card.appendChild(createRow('Type', typeBadge));

    const senderVal = document.createElement('span');
    senderVal.className = 'cm-value';
    senderVal.title = tx.sender || '—';
    senderVal.textContent = truncate(tx.sender);
    card.appendChild(createRow('Sender', senderVal));

    const receiverVal = document.createElement('span');
    receiverVal.className = 'cm-value';
    receiverVal.title = tx.receiver || '—';
    receiverVal.textContent = truncate(tx.receiver);
    card.appendChild(createRow('Receiver', receiverVal));

    if (tx.value) {
      const valueVal = document.createElement('span');
      valueVal.className = 'cm-value cm-highlight';
      valueVal.textContent = formatValue(tx.value, chain);
      card.appendChild(createRow('Value', valueVal));
    }
    body.appendChild(card);

    // Events card (if token transfer)
    if (event.token || event.amount) {
      const evCard = document.createElement('div');
      evCard.className = 'cm-card';
      const evTitle = document.createElement('div');
      evTitle.className = 'cm-card-title';
      evTitle.textContent = 'Transfer Details';
      evCard.appendChild(evTitle);

      if (event.token) {
        const tokenVal = document.createElement('span');
        tokenVal.className = 'cm-value cm-highlight';
        tokenVal.textContent = event.token;
        evCard.appendChild(createRow('Token', tokenVal));
      }
      if (event.amount) {
        const amountVal = document.createElement('span');
        amountVal.className = 'cm-value cm-highlight';
        amountVal.textContent = formatAmount(event.amount, chain);
        evCard.appendChild(createRow('Amount', amountVal));
      }
      if (event.from) {
        const fromVal = document.createElement('span');
        fromVal.className = 'cm-value';
        fromVal.title = event.from;
        fromVal.textContent = truncate(event.from);
        evCard.appendChild(createRow('From', fromVal));
      }
      if (event.to) {
        const toVal = document.createElement('span');
        toVal.className = 'cm-value';
        toVal.title = event.to;
        toVal.textContent = truncate(event.to);
        evCard.appendChild(createRow('To', toVal));
      }
      body.appendChild(evCard);
    }

    // Raw JSON toggle + Copy button row
    const jsonBtnRow = document.createElement('div');
    jsonBtnRow.style.cssText = 'display:flex;gap:6px;margin-top:10px;';

    const rawBtn = document.createElement('button');
    rawBtn.className = 'cm-raw-toggle';
    rawBtn.style.flex = '1';
    rawBtn.textContent = '{ } Show raw JSON';

    const copyBtn = document.createElement('button');
    copyBtn.className = 'cm-raw-toggle';
    copyBtn.style.cssText = 'flex-shrink:0;width:auto;padding:7px 12px;';
    copyBtn.textContent = '⎘ Copy';

    const rawPre = document.createElement('pre');
    rawPre.className = 'cm-raw-json';
    rawPre.textContent = JSON.stringify(tx, null, 2);

    rawBtn.addEventListener('click', () => {
      rawPre.classList.toggle('cm-visible');
      rawBtn.textContent = rawPre.classList.contains('cm-visible')
        ? '{ } Hide raw JSON' : '{ } Show raw JSON';
    });

    copyBtn.addEventListener('click', () => {
      navigator.clipboard.writeText(JSON.stringify(tx, null, 2)).then(() => {
        copyBtn.textContent = '✓ Copied!';
        copyBtn.style.color = '#22d3a0';
        setTimeout(() => {
          copyBtn.textContent = '⎘ Copy';
          copyBtn.style.color = '';
        }, 1500);
      });
    });

    jsonBtnRow.appendChild(rawBtn);
    jsonBtnRow.appendChild(copyBtn);
    body.appendChild(jsonBtnRow);
    body.appendChild(rawPre);

    // AI section placeholder
    const aiSection = document.createElement('div');
    aiSection.id = 'cm-ai-section';
    aiSection.className = 'cm-ai-section';
    aiSection.style.marginTop = '12px';
    body.appendChild(aiSection);
  }

  // ── Render helpers ───────────────────────────────────────────
  function createRow(label, valueNode) {
    const row = document.createElement('div');
    row.className = 'cm-row';
    const labelSpan = document.createElement('span');
    labelSpan.className = 'cm-label';
    labelSpan.textContent = label;
    row.appendChild(labelSpan);
    if (typeof valueNode === 'string') {
      row.appendChild(document.createTextNode(valueNode));
    } else {
      row.appendChild(valueNode);
    }
    return row;
  }

  function formatEventType(t) {
    const map = {
      token_transfer: '🔄 Token Transfer',
      swap:           '🔁 Swap',
      nft_transfer:   '🖼 NFT Transfer',
      stake:          '🔒 Stake',
      bridge:         '🌉 Bridge',
      unsupported:    '⚠ Unsupported',
      unknown:        '❓ Unknown',
    };
    return map[t] || t;
  }

  function formatValue(val, chain) {
    const num = parseFloat(val);
    if (isNaN(num)) return val;

    const config = {
      ethereum: { symbol: 'ETH',  dec: 18 },
      solana:   { symbol: 'SOL',  dec: 9  },
      cosmos:   { symbol: 'ATOM', dec: 6  },
      bitcoin:  { symbol: 'BTC',  dec: 8  },
      starknet: { symbol: 'ETH',  dec: 18 },
    };

    const c = config[chain] || { symbol: '', dec: 0 };
    if (c.dec > 0 && num > 1000) {
      return `${(num / Math.pow(10, c.dec)).toLocaleString(undefined, { maximumFractionDigits: 4 })} ${c.symbol}`;
    }
    return `${num} ${c.symbol}`;
  }

  function formatAmount(val, chain) {
    const num = parseFloat(val);
    if (isNaN(num)) return val;

    // If it looks like a large raw number (wei/lamports), convert it
    // Note: This is an approximation since we don't know the exact token's decimals here
    // but we can guess based on the chain if it's the native asset.
    if (num > 1000000) {
      const dec = chain === 'solana' ? 9 : 18;
      return (num / Math.pow(10, dec)).toLocaleString(undefined, { maximumFractionDigits: 4 });
    }
    return num.toLocaleString(undefined, { maximumFractionDigits: 6 });
  }

  function detectAlerts(tx, event) {
    const alerts = [];
    // Whale detection: check native value
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
      alerts.push({ type: 'risk', msg: `⚠ Zero-value contract interaction — verify before trusting` });
    }
    // Risk: unsupported event
    if (event.event_type === 'unsupported') {
      alerts.push({ type: 'risk', msg: `⚠ Complex transaction — could not fully decode all events` });
    }
    return alerts;
  }

  // ── AI Section Helpers ───────────────────────────────────────
  function getAiSection() { return document.getElementById('cm-ai-section'); }

  function renderAiLoading(body) {
    const sec = getAiSection();
    if (!sec) return;
    sec.textContent = '';
    
    const card = document.createElement('div');
    card.className = 'cm-card';
    
    const title = document.createElement('div');
    title.className = 'cm-ai-title';
    title.textContent = '✨ AI Explanation ';
    const badge = document.createElement('span');
    badge.className = 'cm-ai-badge';
    badge.textContent = 'Gemini';
    title.appendChild(badge);
    
    const loading = document.createElement('div');
    loading.className = 'cm-ai-loading';
    const spinner = document.createElement('div');
    spinner.className = 'cm-spinner';
    spinner.style.width = '14px';
    spinner.style.height = '14px';
    spinner.style.borderWidth = '2px';
    const text = document.createElement('span');
    text.textContent = 'Generating explanation…';
    
    loading.appendChild(spinner);
    loading.appendChild(text);
    card.appendChild(title);
    card.appendChild(loading);
    sec.appendChild(card);
  }

  function renderAiExplanation(body, { text, thought }) {
    const sec = getAiSection();
    if (!sec) return;
    sec.textContent = '';

    const card = document.createElement('div');
    card.className = 'cm-card';
    
    const title = document.createElement('div');
    title.className = 'cm-ai-title';
    title.textContent = '✨ Forensic Report ';
    const badge = document.createElement('span');
    badge.className = 'cm-ai-badge';
    badge.textContent = 'Deep Think';
    title.appendChild(badge);

    if (thought) {
      const reasoning = document.createElement('div');
      reasoning.className = 'cm-ai-reasoning';
      reasoning.style.fontSize = '10px';
      reasoning.style.color = '#8a97b0';
      reasoning.style.margin = '8px 0';
      reasoning.style.padding = '8px';
      reasoning.style.background = 'rgba(255,255,255,0.03)';
      reasoning.style.borderLeft = '2px solid var(--accent)';
      reasoning.style.fontFamily = 'monospace';
      reasoning.style.maxHeight = '100px';
      reasoning.style.overflowY = 'auto';
      reasoning.textContent = thought;
      card.appendChild(reasoning);
    }

    const content = document.createElement('div');
    content.className = 'cm-ai-text';
    content.textContent = text;

    card.appendChild(title);
    card.appendChild(content);
    sec.appendChild(card);
  }

  function renderAiError(body, msg) {
    const sec = getAiSection();
    if (!sec) return;
    sec.textContent = '';

    const card = document.createElement('div');
    card.className = 'cm-card';
    
    const title = document.createElement('div');
    title.className = 'cm-ai-title';
    title.textContent = '✨ AI Explanation ';
    const badge = document.createElement('span');
    badge.className = 'cm-ai-badge';
    badge.textContent = 'Gemini';
    title.appendChild(badge);

    const content = document.createElement('div');
    content.className = 'cm-ai-text';
    content.style.color = '#f05b6e';
    content.style.fontSize = '11px';
    content.textContent = `Could not generate explanation: ${msg}`;

    card.appendChild(title);
    card.appendChild(content);
    sec.appendChild(card);
  }

  function renderAiNoKey(body) {
    const sec = getAiSection();
    if (!sec) return;
    sec.textContent = '';

    const card = document.createElement('div');
    card.className = 'cm-card';
    card.style.textAlign = 'center';
    card.style.padding = '12px';
    
    const content = document.createElement('div');
    content.style.color = '#8a97b0';
    content.style.fontSize = '11px';
    content.style.lineHeight = '1.6';

    const text1 = document.createTextNode('✨ ');
    const strong = document.createElement('strong');
    strong.style.color = '#e8edf8';
    strong.textContent = 'AI Explanations';
    const text2 = document.createTextNode(' available');
    const br = document.createElement('br');
    const text3 = document.createTextNode('Add your Gemini API key in the extension popup.');

    content.appendChild(text1);
    content.appendChild(strong);
    content.appendChild(text2);
    content.appendChild(br);
    content.appendChild(text3);
    
    card.appendChild(content);
    sec.appendChild(card);
  }

  // ── Error State ───────────────────────────────────────────────
  function renderError(body, msg, apiBase) {
    const isOffline = msg.includes('fetch') || msg.includes('Failed') || msg.includes('NetworkError');
    body.textContent = '';

    const errorDiv = document.createElement('div');
    errorDiv.className = 'cm-error';
    
    const icon = document.createElement('div');
    icon.className = 'cm-error-icon';
    icon.textContent = isOffline ? '🔌' : '⚠';
    
    const msgDiv = document.createElement('div');
    msgDiv.className = 'cm-error-msg';
    msgDiv.textContent = msg;

    const subDiv = document.createElement('div');
    subDiv.className = 'cm-error-sub';
    if (isOffline) {
      subDiv.appendChild(document.createTextNode('Make sure the ChainMerge API is running at '));
      const code = document.createElement('code');
      code.textContent = apiBase;
      subDiv.appendChild(code);
    } else {
      subDiv.textContent = 'This transaction type may not be supported yet.';
    }

    errorDiv.appendChild(icon);
    errorDiv.appendChild(msgDiv);
    errorDiv.appendChild(subDiv);
    body.appendChild(errorDiv);
  }

  // ── Gemini AI ────────────────────────────────────────────────
  async function fetchGeminiExplanation(apiKey, tx, attempt = 1) {
    const event = tx.events?.[0] || {};
    const prompt = buildPrompt(tx, event);

    const res = await fetch(`${GEMINI_API_BASE}/key=${apiKey}`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      signal: AbortSignal.timeout(30000),
      body: JSON.stringify({
        contents: [{ parts: [{ text: prompt }] }],
        generationConfig: { 
          temperature: 0.4, 
          maxOutputTokens: 1000,
          topP: 0.95
        },
      }),
    });

    // 429 = transient rate limit — retry once after 26 seconds
    if (res.status === 429 && attempt === 1) {
      const aiSec = getAiSection();
      if (aiSec) {
        aiSec.querySelector('.cm-ai-loading') &&
          (aiSec.querySelector('.cm-ai-loading').textContent = '⏳ Rate limited — retrying in 26s…');
      }
      await new Promise((r) => setTimeout(r, 26000));
      return fetchGeminiExplanation(apiKey, tx, 2);
    }

    if (!res.ok) {
      const err = await res.json().catch(() => ({}));
      throw new Error(err.error?.message || `Gemini API error ${res.status}`);
    }

    const data = await res.json();
    const candidate = data.candidates?.[0]?.content?.parts || [];
    
    let thought = "";
    let text = "";

    candidate.forEach(p => {
      if (p.thought) thought += p.thought;
      if (p.text) text += p.text;
    });

    if (!text && candidate[0]?.text) text = candidate[0].text;

    return { text: text?.trim() || 'No explanation', thought: thought.trim() };
  }

  function buildPrompt(tx, event) {
    return `Perform a deep forensic analysis of this blockchain transaction. 
Identify the likely intent (e.g., Arbitrage, Token Swap, Whale Movement, Phishing, Bridge).

Provide a concise forensic report in maximum 2 clear, simple sentences for a non-technical user.
CRITICAL: You MUST provide a complete explanation and finish every sentence. NEVER stop mid-sentence.
Keep the final output under 400 characters.

Transaction Data:
Chain: ${tx.chain}
Hash: ${tx.tx_hash}
Sender: ${tx.sender || 'unknown'}
Receiver: ${tx.receiver || 'unknown'}
Value: ${tx.value || '0'}
Type: ${event.event_type || 'transaction'}
${event.token ? `Token: ${event.token}` : ''}
${event.amount ? `Amount: ${event.amount}` : ''}

Respond only with the plain English report.`;
  }

  // ── Clipboard Tracking ──────────────────────────────────────
  const HASH_REGEX = {
    ethereum: /0x[a-fA-F0-9]{64}/,
    solana:   /[1-9A-HJ-NP-Za-km-z]{32,88}/,
  };

  document.addEventListener('copy', () => {
    // Small delay to let the clipboard update
    setTimeout(async () => {
      try {
        const text = await navigator.clipboard.readText();
        const val = text.trim();
        if (!val) return;

        let detectedChain = null;
        if (HASH_REGEX.ethereum.test(val)) detectedChain = 'ethereum';
        else if (HASH_REGEX.solana.test(val)) detectedChain = 'solana';

        if (detectedChain) {
          showCmToast(`ChainMerge: ${detectedChain} hash captured!`);
          chrome.storage.local.set({ lastCapturedHash: { chain: detectedChain, hash: val, time: Date.now() } });
        }
      } catch (err) {}
    }, 100);
  });

  function showCmToast(msg) {
    const toast = document.createElement('div');
    toast.className = 'cm-toast-popup';
    
    const icon = document.createElement('span');
    icon.style.fontSize = '16px';
    icon.textContent = '⛓';
    
    const text = document.createElement('span');
    text.textContent = msg;

    toast.appendChild(icon);
    toast.appendChild(text);
    
    document.body.appendChild(toast);
    setTimeout(() => toast.classList.add('visible'), 100);
    setTimeout(() => {
      toast.classList.remove('visible');
      setTimeout(() => toast.remove(), 400);
    }, 3000);
  }

  // ── Utils ────────────────────────────────────────────────────
  function escapeHtml(str) {
    return String(str)
      .replace(/&/g, '&amp;')
      .replace(/</g, '&lt;')
      .replace(/>/g, '&gt;')
      .replace(/"/g, '&quot;');
  }
})();
