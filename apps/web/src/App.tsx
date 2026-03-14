import { FormEvent, useMemo, useState, useCallback } from "react";

// ── Types ──────────────────────────────────────────────────────
type DecodeSuccess = {
  decoded: {
    chain: string;
    tx_hash: string;
    sender?: string;
    receiver?: string;
    value?: string;
    events: Array<{
      event_type: string;
      token?: string;
      from?: string;
      to?: string;
      amount?: string;
      raw_program?: string;
    }>;
  };
};



type DecodeFailure = {
  error: {
    code: string;
    message: string;
    retryable: boolean;
  };
};

// ── Chain Config ───────────────────────────────────────────────
const CHAIN_OPTIONS = [
  "ethereum",
  "solana",
  "cosmos",
  "aptos",
  "sui",
  "polkadot",
  "bitcoin",
  "starknet",
] as const;

const CHAIN_EMOJI: Record<string, string> = {
  ethereum:  "⟠",
  solana:    "◎",
  cosmos:    "⚛",
  aptos:     "▲",
  sui:       "💧",
  polkadot:  "●",
  bitcoin:   "₿",
  starknet:  "★",
};

// ── JSON Syntax Highlighter ────────────────────────────────────
function syntaxHighlight(json: string): string {
  return json
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(
      /("(\\u[\da-fA-F]{4}|\\[^u]|[^\\"])*"(\s*:)?|\b(true|false|null)\b|-?\d+\.?\d*([eE][+-]?\d+)?)/g,
      (match) => {
        if (/^"/.test(match)) {
          if (/:$/.test(match)) {
            return `<span class="json-key">${match}</span>`;
          }
          return `<span class="json-str">${match}</span>`;
        }
        if (/true|false/.test(match)) return `<span class="json-bool">${match}</span>`;
        if (/null/.test(match)) return `<span class="json-null">${match}</span>`;
        return `<span class="json-num">${match}</span>`;
      }
    );
}

// ── App ────────────────────────────────────────────────────────
export function App() {
  const [chain, setChain] = useState<(typeof CHAIN_OPTIONS)[number]>("ethereum");
  const [hash, setHash] = useState(
    "0xd5d0587189f3411699ae946baa2a7d3ebfaf13133f9014a22bab6948591611ad"
  );
  const [rpcUrl, setRpcUrl]   = useState("");
  const [loading, setLoading] = useState(false);
  const [response, setResponse] = useState<DecodeSuccess | null>(null);
  const [error, setError]       = useState<DecodeFailure | null>(null);
  const [copied, setCopied]     = useState(false);

  const sampleTip = useMemo(() => {
    if (chain === "ethereum") return "Use presets below for verified ERC-20 and native ETH examples.";
    if (chain === "solana")   return "Use an SPL transfer hash to get token_transfer output.";
    if (chain === "cosmos")   return "Cosmos decoder supports bank MsgSend via Cosmos tx REST endpoint.";
    return "This chain key exists, but decoding may still be placeholder.";
  }, [chain]);

  function applyPreset(kind: "eth_erc20" | "eth_native") {
    setChain("ethereum");
    setRpcUrl("");
    setHash(
      kind === "eth_erc20"
        ? "0xd5d0587189f3411699ae946baa2a7d3ebfaf13133f9014a22bab6948591611ad"
        : "0x5db45209923531658781b4a5ea73bde7193e7f0991595ad5af80121764afb8b4"
    );
  }

  async function onSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    setLoading(true);
    setError(null);
    setResponse(null);

    try {
      const params = new URLSearchParams({ chain, hash: hash.trim() });
      if (rpcUrl.trim()) params.set("rpc_url", rpcUrl.trim());

      const res  = await fetch(`/api/decode?${params.toString()}`);
      const body = (await res.json()) as DecodeSuccess | DecodeFailure;

      if (!res.ok) { setError(body as DecodeFailure); return; }
      const decoded = (body as DecodeSuccess).decoded;
      setResponse(body as DecodeSuccess);
      
    } catch (err) {
      setError({
        error: {
          code: "network_error",
          message: err instanceof Error ? err.message : "Unknown network error",
          retryable: true,
        },
      });
    } finally {
      setLoading(false);
    }
  }

  const handleCopy = useCallback(() => {
    const text = response
      ? JSON.stringify(response, null, 2)
      : error
      ? JSON.stringify(error, null, 2)
      : "";
    if (!text) return;
    navigator.clipboard.writeText(text).then(() => {
      setCopied(true);
      setTimeout(() => setCopied(false), 1800);
    });
  }, [response, error]);

  const hasOutput = !!(response || error);
  const outputJson = response
    ? JSON.stringify(response, null, 2)
    : error
    ? JSON.stringify(error, null, 2)
    : "";

  return (
    <main className="page">
      {/* ── Hero ── */}
      <section className="panel hero">
        <div className="hero-orb hero-orb-1" />
        <div className="hero-orb hero-orb-2" />
        <p className="eyebrow">
          <span className="eyebrow-dot" />
          ChainMerge · Decoder Playground
        </p>
        <h1>Multichain Transaction Decoder</h1>
        <p className="subtitle">
          Submit any transaction hash and receive a single, normalized JSON response —
          powered by a high-performance Rust backend.
        </p>
      </section>

      {/* ── Form Panel ── */}
      <section className="panel form-panel">
        <form onSubmit={onSubmit} className="decode-form">

          <label>
            Chain
            <div className="chain-select-wrap">
              <span className="chain-emoji-label">{CHAIN_EMOJI[chain]}</span>
              <select
                value={chain}
                onChange={(e) => setChain(e.target.value as (typeof CHAIN_OPTIONS)[number])}
              >
                {CHAIN_OPTIONS.map((opt) => (
                  <option key={opt} value={opt}>
                    {opt.charAt(0).toUpperCase() + opt.slice(1)}
                  </option>
                ))}
              </select>
            </div>
          </label>

          <label>
            Transaction Hash
            <input
              value={hash}
              onChange={(e) => setHash(e.target.value)}
              placeholder="0x… or base58 hash"
              required
              spellCheck={false}
              autoCorrect="off"
            />
          </label>

          <label>
            Custom RPC URL
            <input
              value={rpcUrl}
              onChange={(e) => setRpcUrl(e.target.value)}
              placeholder="https://…  (optional — built-in defaults apply)"
            />
          </label>

          <button type="submit" className="btn-primary" disabled={loading}>
            {loading ? (
              <>
                <span className="spinner" />
                Decoding…
              </>
            ) : (
              <>⚡ Decode Transaction</>
            )}
          </button>
        </form>

        <div className="presets">
          <p className="preset-title">Quick Presets</p>
          <div className="preset-actions">
            <button type="button" className="ghost" onClick={() => applyPreset("eth_erc20")}>
              ⟠ ETH — ERC-20 Transfer
            </button>
            <button type="button" className="ghost" onClick={() => applyPreset("eth_native")}>
              ⟠ ETH — Native Transfer
            </button>
          </div>
        </div>

        <p className="tip">{sampleTip}</p>
      </section>

      {/* ── Output Panel ── */}
      <section className="panel output-panel">
        <div className="output-panel-header">
          <h2>Output</h2>
          <div style={{ display: "flex", alignItems: "center", gap: "0.6rem" }}>
            {response && (
              <span className="status-badge success">
                ✓ Success
              </span>
            )}
            {error && (
              <span className="status-badge error">
                ✕ Error
              </span>
            )}
            {hasOutput && (
              <button className="copy-btn" onClick={handleCopy}>
                {copied ? "✓ Copied!" : "Copy"}
              </button>
            )}
          </div>
        </div>

        {!hasOutput && (
          <div className="output-placeholder">
            <span className="output-icon">⛓</span>
            <span>Run a decode request to see the normalized JSON response here.</span>
          </div>
        )}


        {hasOutput && (
          <div className="output-block">
            <pre
              className={`output ${response ? "success" : "error"}`}
              dangerouslySetInnerHTML={{ __html: syntaxHighlight(outputJson) }}
            />
          </div>
        )}
      </section>
    </main>
  );
}
