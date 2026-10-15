/**
 * Storeback Protocol Client (TypeScript)
 * Enables autonomous machine-to-machine micropayments and sovereign data vending.
 */

import { X402Challenge, X402Proof, StorebackResponse } from "./types";

export interface StorebackClientConfig {
  xrplSecret?: string;
  basePrivateKey?: string;
  timeoutMs?: number;
}

export class StorebackClient {
  private config: StorebackClientConfig;

  constructor(config: StorebackClientConfig = {}) {
    this.config = config;
  }

  /**
   * Probes endpoint to extract RFC-0402 HTTP challenge
   */
  async getChallenge(endpointUrl: string): Promise<X402Challenge> {
    const res = await fetch(endpointUrl, { method: "GET" });
    if (res.status !== 402) {
      throw new Error(`Expected HTTP 402 Payment Required, received ${res.status}`);
    }

    const h = res.headers;
    const drops = parseInt(h.get("x-402-price-drops") || "0", 10);
    const xrp = parseFloat(h.get("x-402-price-xrp") || (drops / 1_000_000).toString());
    const micros = parseInt(h.get("x-402-price-usdc-micros") || "0", 10);
    const usdc = parseFloat(h.get("x-402-price-usdc") || (micros / 1_000_000).toString());

    return {
      version: h.get("x-402-version") || "1.0",
      realm: (h.get("www-authenticate") || "Storeback").replace(/x402 realm="/i, "").replace(/"$/, ""),
      slug: h.get("x-402-slug") || "",
      priceDrops: drops,
      priceXrp: xrp,
      priceUsdcMicros: micros,
      priceUsdc: usdc,
      recipientXrpl: h.get("x-402-recipient-xrpl") || "",
      recipientBaseUsdc: h.get("x-402-recipient-base-usdc") || "",
      nonce: h.get("x-402-nonce") || "",
      expiresAt: parseInt(h.get("x-402-expires") || "0", 10),
      digest: h.get("x-402-digest") || undefined,
    };
  }

  /**
   * Queries protected endpoint with verified payment proof
   */
  async queryWithProof<T = any>(
    endpointUrl: string,
    proof: X402Proof,
    options: { useXPayment?: boolean } = {}
  ): Promise<StorebackResponse<T>> {
    const headers: Record<string, string> = {};

    if (options.useXPayment) {
      const payload = {
        chain: proof.chain,
        tx_hash: proof.txHash,
        nonce: proof.nonce,
      };
      headers["X-PAYMENT"] = btoa(JSON.stringify(payload));
    } else {
      headers["Authorization"] = `x402 tx_hash=${proof.txHash} nonce=${proof.nonce} chain=${proof.chain}`;
    }

    const res = await fetch(endpointUrl, {
      method: "GET",
      headers,
    });

    if (res.status === 200) {
      return (await res.json()) as StorebackResponse<T>;
    } else if (res.status === 402) {
      const body = await res.json().catch(() => ({}));
      throw new Error(`HTTP 402 Payment Failed: ${body.verification_error || "Invalid proof"}`);
    } else {
      throw new Error(`HTTP Request failed with status ${res.status}: ${res.statusText}`);
    }
  }
}
