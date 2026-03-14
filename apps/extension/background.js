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
const HARDCODED_GEMINI_KEY = "AIzaSyDExbWVlYOyX0J3zEZ0aFR4K45qk1-Vsms";

_chr.runtime.onMessage.addListener((message, sender, sendResponse) => {
  if (message.type === 'CM_DECODE') {
    handleDecode(message).then(sendResponse).catch((err) => {
      sendResponse({ ok: false, error: err.message });
    });
    return true; 
  }

  if (message.type === 'CM_GEMINI_EXPLAIN') {
    handleGeminiExplain(message, HARDCODED_GEMINI_KEY).then(sendResponse).catch((err) => {
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

async function handleGeminiExplain({ decoded }, apiKey) {
  const prompt = buildPrompt(decoded);
  try {
    const res = await fetch(`${GEMINI_API_BASE}?key=${apiKey}`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        contents: [{ parts: [{ text: prompt }] }],
        generationConfig: { 
          temperature: 0.4, 
          maxOutputTokens: 1000,
          topP: 0.95
        }
      }),
      signal: AbortSignal.timeout(30000)
    });

    if (!res.ok) {
      const err = await res.json().catch(() => ({}));
      throw new Error(err.error?.message || `Gemini error ${res.status}`);
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

    // Post-process text
    let cleaned = text?.trim() || 'No explanation generated.';
    if (cleaned !== 'No explanation generated.' && !cleaned.endsWith('.') && !cleaned.endsWith('!') && !cleaned.endsWith('?')) {
        const lastPunc = Math.max(cleaned.lastIndexOf('.'), cleaned.lastIndexOf('!'), cleaned.lastIndexOf('?'));
        if (lastPunc > 0) cleaned = cleaned.substring(0, lastPunc + 1);
    }

    return { ok: true, text: cleaned, thought: thought.trim() };
  } catch (err) {
    return { ok: false, error: err.message };
  }
}

function buildPrompt(tx) {
  const event = tx.events?.[0] || {};
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
