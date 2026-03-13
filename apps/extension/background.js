/* background.js — ChainMerge Service Worker
 *
 * Content scripts on public pages (etherscan.io etc.) cannot fetch localhost
 * due to Chrome's Private Network Access policy. This service worker acts
 * as a proxy: content.js sends a message here, we fetch localhost, reply.
 */

/* Firefox/Chrome compatibility shim */
const _chr = (typeof browser !== 'undefined' && browser.runtime) ? browser : chrome;

const DEFAULT_API_URL = 'http://localhost:8080';
const GEMINI_API_BASE = 'https://generativelanguage.googleapis.com/v1beta/models/gemini-2.5-flash:generateContent';

_chr.runtime.onMessage.addListener((message, sender, sendResponse) => {
  if (message.type === 'CM_DECODE') {
    handleDecode(message).then(sendResponse).catch((err) => {
      sendResponse({ ok: false, error: err.message });
    });
    return true; 
  }

  if (message.type === 'CM_GEMINI_EXPLAIN') {
    handleGeminiExplain(message).then(sendResponse).catch((err) => {
      sendResponse({ ok: false, error: err.message });
    });
    return true;
  }

  if (message.type === 'CM_HEALTH') {
    handleHealth(message).then(sendResponse).catch(() => {
      sendResponse({ ok: false });
    });
    return true;
  }
});

async function handleDecode({ chain, hash, apiUrl, apiKey }) {
  const base   = (apiUrl || DEFAULT_API_URL).replace(/\/+$/, '');
  const params = new URLSearchParams({ chain, hash });
  const url    = `${base}/api/decode?${params}`;

  const headers = {};
  if (apiKey) headers['x-api-key'] = apiKey;

  try {
    const res  = await fetch(url, { headers, signal: AbortSignal.timeout(15000) });
    const body = await res.json();
    if (!res.ok) return { ok: false, error: body.error?.message || 'Decode failed' };
    return { ok: true, decoded: body.decoded };
  } catch (err) {
    return { ok: false, error: err.message };
  }
}

async function handleHealth({ apiUrl, apiKey }) {
  const base = (apiUrl || DEFAULT_API_URL).replace(/\/+$/, '');
  const headers = {};
  if (apiKey) headers['x-api-key'] = apiKey;

  try {
    const res  = await fetch(`${base}/api/health`, { headers, signal: AbortSignal.timeout(3000) });
    const body = await res.json();
    return { ok: res.ok && body.status === 'ok' };
  } catch {
    return { ok: false };
  }
}

async function handleGeminiExplain({ apiKey, decoded }) {
  const prompt = buildPrompt(decoded);
  try {
    const res = await fetch(`${GEMINI_API_BASE}?key=${apiKey}`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        contents: [{ parts: [{ text: prompt }] }],
        generationConfig: { temperature: 0.3, maxOutputTokens: 600 }
      }),
      signal: AbortSignal.timeout(30000)
    });

    if (!res.ok) {
      const err = await res.json().catch(() => ({}));
      throw new Error(err.error?.message || `Gemini error ${res.status}`);
    }
    
    const data = await res.json();
    const text = data.candidates?.[0]?.content?.parts?.[0]?.text;
    return { ok: true, text: text?.trim() || 'No explanation generated.' };
  } catch (err) {
    return { ok: false, error: err.message };
  }
}

function buildPrompt(tx) {
  const event = tx.events?.[0] || {};
  function truncate(s, n=10) { return s && s.length > n*2+3 ? `${s.slice(0,n)}...${s.slice(-n)}` : s; }

  return `Explain this blockchain transaction in 2 simple sentences for a non-technical user.
Chain: ${tx.chain}
Hash: ${truncate(tx.tx_hash)}
Sender: ${truncate(tx.sender) || 'unknown'}
Receiver: ${truncate(tx.receiver) || 'unknown'}
Value: ${tx.value || '0'}
Type: ${event.event_type || 'transaction'}
${event.token ? `Token: ${event.token}` : ''}
${event.amount ? `Amount: ${event.amount}` : ''}
Respond only with the plain English explanation.`;
}
