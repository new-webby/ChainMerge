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
  toggle.innerHTML = `<span class="cm-toggle-icon">⛓</span> CM`;
  toggle.title = 'Toggle ChainMerge Panel';
  document.body.appendChild(toggle);

  // Panel container
  const panel = document.createElement('div');
  panel.className = 'cm-panel';
  panel.innerHTML = `
    <div class="cm-header">
      <div class="cm-header-left">
        <span style="font-size:18px;">⛓</span>
        <span class="cm-logo-text">ChainMerge</span>
      </div>
      <div class="cm-header-right">
        <button class="cm-close-btn" id="cm-close">✕</button>
      </div>
    </div>
    <div class="cm-body" id="cm-body">
      <div class="cm-loading">
        <div class="cm-spinner"></div>
        <span>Decoding transaction…</span>
      </div>
    </div>
    <div class="cm-footer">
      <span class="cm-footer-brand">Powered by ChainMerge SDK</span>
      <a class="cm-footer-link" href="http://localhost:5173" target="_blank">Open Playground →</a>
    </div>
  `;
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
        const explanation = await fetchGeminiExplanation(geminiApiKey, decoded);
        renderAiExplanation(body, explanation);
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

    body.innerHTML = '';

    // Chain + hash
    const chainRow = document.createElement('div');
    chainRow.className = 'cm-chain-row';
    chainRow.innerHTML = `
      <span class="cm-chain-badge">${emoji} ${chain}</span>
    `;
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
    card.innerHTML = `
      <div class="cm-card-title">Transaction</div>
      ${row('Type', `<span class="cm-event-badge cm-event-${eventType}">${formatEventType(eventType)}</span>`)}
      ${row('Sender', `<span class="cm-value" title="${tx.sender || '—'}">${truncate(tx.sender)}</span>`)}
      ${row('Receiver', `<span class="cm-value" title="${tx.receiver || '—'}">${truncate(tx.receiver)}</span>`)}
      ${tx.value ? row('Value', `<span class="cm-value cm-highlight">${formatValue(tx.value, chain)}</span>`) : ''}
    `;
    body.appendChild(card);

    // Events card (if token transfer)
    if (event.token || event.amount) {
      const evCard = document.createElement('div');
      evCard.className = 'cm-card';
      evCard.innerHTML = `
        <div class="cm-card-title">Transfer Details</div>
        ${event.token  ? row('Token',  `<span class="cm-value cm-highlight">${event.token}</span>`) : ''}
        ${event.amount ? row('Amount', `<span class="cm-value cm-highlight">${formatAmount(event.amount, chain)}</span>`) : ''}
        ${event.from   ? row('From',   `<span class="cm-value" title="${event.from}">${truncate(event.from)}</span>`) : ''}
        ${event.to     ? row('To',     `<span class="cm-value" title="${event.to}">${truncate(event.to)}</span>`) : ''}
      `;
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
  function row(label, valueHtml) {
    return `
      <div class="cm-row">
        <span class="cm-label">${label}</span>
        ${valueHtml}
      </div>`;
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
    sec.innerHTML = `
      <div class="cm-card">
        <div class="cm-ai-title">
          ✨ AI Explanation
          <span class="cm-ai-badge">Gemini</span>
        </div>
        <div class="cm-ai-loading">
          <div class="cm-spinner" style="width:14px;height:14px;border-width:2px;"></div>
          Generating explanation…
        </div>
      </div>`;
  }

  function renderAiExplanation(body, text) {
    const sec = getAiSection();
    if (!sec) return;
    sec.innerHTML = `
      <div class="cm-card">
        <div class="cm-ai-title">
          ✨ AI Explanation
          <span class="cm-ai-badge">Gemini</span>
        </div>
        <div class="cm-ai-text">${escapeHtml(text)}</div>
      </div>`;
  }

  function renderAiError(body, msg) {
    const sec = getAiSection();
    if (!sec) return;
    sec.innerHTML = `
      <div class="cm-card">
        <div class="cm-ai-title">
          ✨ AI Explanation
          <span class="cm-ai-badge">Gemini</span>
        </div>
        <div class="cm-ai-text" style="color:#f05b6e;font-size:11px;">
          Could not generate explanation: ${escapeHtml(msg)}
        </div>
      </div>`;
  }

  function renderAiNoKey(body) {
    const sec = getAiSection();
    if (!sec) return;
    sec.innerHTML = `
      <div class="cm-card" style="text-align:center;padding:12px;">
        <div style="color:#8a97b0;font-size:11px;line-height:1.6;">
          ✨ <strong style="color:#e8edf8;">AI Explanations</strong> available<br>
          Add your Gemini API key in the extension popup.
        </div>
      </div>`;
  }

  // ── Error State ───────────────────────────────────────────────
  function renderError(body, msg, apiBase) {
    const isOffline = msg.includes('fetch') || msg.includes('Failed') || msg.includes('NetworkError');
    body.innerHTML = `
      <div class="cm-error">
        <div class="cm-error-icon">${isOffline ? '🔌' : '⚠'}</div>
        <div class="cm-error-msg">${escapeHtml(msg)}</div>
        <div class="cm-error-sub">${
          isOffline
            ? `Make sure the ChainMerge API is running at <code>${apiBase}</code>`
            : 'This transaction type may not be supported yet.'
        }</div>
      </div>`;
  }

  // ── Gemini AI ────────────────────────────────────────────────
  async function fetchGeminiExplanation(apiKey, tx, attempt = 1) {
    const event = tx.events?.[0] || {};
    const prompt = buildPrompt(tx, event);

    const res = await fetch(`${GEMINI_API_BASE}?key=${apiKey}`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      signal: AbortSignal.timeout(15000),
      body: JSON.stringify({
        contents: [{ parts: [{ text: prompt }] }],
        generationConfig: { temperature: 0.3, maxOutputTokens: 700 },
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
    const text = data.candidates?.[0]?.content?.parts?.[0]?.text;
    if (!text) throw new Error('Empty response from Gemini');
    return text.trim();
  }

  function buildPrompt(tx, event) {
    const lines = [
      `Explain this blockchain transaction in 2-3 simple sentences for a non-technical user.`,
      `Be concise, friendly, and use plain English. Do not use technical jargon.`,
      ``,
      `Chain: ${tx.chain}`,
      `Transaction hash: ${truncate(tx.tx_hash, 10)}`,
    ];

    if (tx.sender)   lines.push(`Sender: ${tx.sender}`);
    if (tx.receiver) lines.push(`Receiver: ${tx.receiver}`);
    if (tx.value && tx.value !== '0') lines.push(`Value: ${tx.value}`);

    if (event.event_type) lines.push(`Event type: ${event.event_type}`);
    if (event.token)      lines.push(`Token: ${event.token}`);
    if (event.amount)     lines.push(`Amount: ${event.amount}`);
    if (event.from)       lines.push(`From: ${event.from}`);
    if (event.to)         lines.push(`To: ${event.to}`);

    lines.push(``, `Respond with only the plain explanation — no formatting, no markdown, no JSON.`);
    return lines.join('\n');
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
