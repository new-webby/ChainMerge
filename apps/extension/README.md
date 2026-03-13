# ChainMerge Browser Extension

A Chrome extension that automatically decodes blockchain transactions on block explorers, powered by the **ChainMerge Rust API** and **Gemini AI**.

## Loading the Extension in Chrome

1. Open Chrome and go to `chrome://extensions`
2. Enable **Developer mode** (top-right toggle)
3. Click **Load unpacked**
4. Select the `apps/extension/` folder from this repo
5. The ChainMerge icon appears in your Chrome toolbar

## Setup

1. Click the **ChainMerge toolbar icon** to open settings
2. Paste your **Gemini API key** (`AIza…`) and click **Save Key**
3. Make sure the **ChainMerge API is running** (`cargo run` in `services/api`)

## Usage

Visit any supported block explorer transaction page:

| Explorer | Chain |
|---|---|
| `etherscan.io/tx/…` | Ethereum |
| `solscan.io/tx/…` | Solana |
| `solana.fm/tx/…` | Solana |
| `starkscan.co/tx/…` | StarkNet |
| `mintscan.io/*/txs/…` | Cosmos |
| `arbiscan.io/tx/…` | Ethereum |

The ChainMerge panel slides in from the right automatically, showing:
- **Decoded transaction** (sender, receiver, token, amount)
- **🐋 Whale alerts** for large transfers
- **⚠ Risk flags** for contract interactions
- **✨ Gemini AI explanation** in plain English

## Files

```
apps/extension/
├── manifest.json    # Manifest V3 config
├── content.js       # Page injection: detect hash, decode, display panel
├── panel.css        # Glassmorphism panel styles
├── popup.html       # Settings popup UI
├── popup.js         # Settings popup logic
└── icons/           # Extension icons (16, 48, 128px)
```
