# Storeback Protocol (`x402`)

> **Autonomous Machine-to-Machine Knowledge Vending & Micropayment Settlement over HTTP 402 and XRPL**

[![Protocol Version](https://img.shields.io/badge/protocol-x402%20v1.0-blue.svg)](https://storeback.org)
[![License](https://img.shields.io/badge/license-Apache%202.0-green.svg)](LICENSE)
[![Network](https://img.shields.io/badge/relay-storeback.io-orange.svg)](https://storeback.io)

Storeback standardizes how sovereign edge databases, local-first vaults, and AI model agents monetize structured private knowledge without central cloud custody, open firewall ports, or data leakage.

---

## ⚡ Core Concept: The Outbound Vending Machine

While existing data platforms require users to upload confidential documents to third-party multitenant clouds (where they can be scraped, leaked, or subpoenaed), **Storeback** turns any local device into an outbound-only vending machine:

1. **Autonomous Machine-to-Machine Negotiation**: External AI agents (Claude, OpenAI Swarms, autonomous procurement bots) query protected datasets.
2. **Standardized HTTP 402 Challenge**: The node responds with `HTTP 402 Payment Required`, challenge nonces, and drop pricing.
3. **Instant Micro-Settlement**: The buyer signs a micro-payment on the XRP Ledger (XRPL).
4. **In-Memory Differential Privacy**: The host mathematically redacts PII and adds calibrated Laplacian noise ($\varepsilon=1.0$) in volatile RAM before streaming synthetic JSON.
5. **Zero Cloud Custody**: Raw databases never leave the sovereign host.

---

## 📖 Specifications & Schemas
- [RFC-0402: HTTP 402 Micropayment Rails](RFC-0402.md)
- [Zero-Knowledge Directory Manifest Schema (`schemas/datacatalog.json`)](schemas/datacatalog.json)

---

## 📦 Client SDKs

### Python
```bash
pip install storeback
```
```python
from storeback import StorebackClient

client = StorebackClient(xrpl_seed="sEd...")
data = client.query("https://relay.storeback.io/node/v-8f2a/query?slug=defense-metals-2026")
print(data)
```

### TypeScript / Node.js
```bash
npm install @storeback/client
```
```typescript
import { StorebackClient } from "@storeback/client";

const client = new StorebackClient({ xrplSecret: "sEd..." });
const response = await client.query("https://relay.storeback.io/node/v-8f2a/query?slug=defense-metals-2026");
console.log(response);
```

---

## 📜 Governance & License
The Storeback wire protocol, schemas, and client SDKs are released under the [Apache 2.0 License](LICENSE).
