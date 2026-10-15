/**
 * Storeback Protocol RFC-0402 Types
 */

export type PaymentChain = "xrpl" | "base";

export interface X402Challenge {
  version: string;
  realm: string;
  slug: string;
  priceDrops: number;
  priceXrp: number;
  priceUsdcMicros: number;
  priceUsdc: number;
  recipientXrpl: string;
  recipientBaseUsdc: string;
  nonce: string;
  expiresAt: number;
  digest?: string;
}

export interface X402Proof {
  chain: PaymentChain;
  txHash: string;
  nonce: string;
  amountDrops?: number;
  amountUsdcMicros?: number;
  payerAddress?: string;
}

export interface StorebackResponse<T = any> {
  status: "success" | "error";
  resource_slug: string;
  title: string;
  data_source: "live_vault" | "synthetic_seed";
  privacy_standard: {
    engine: string;
    epsilon: number;
    deterministic_noise: boolean;
    tqi: {
      tqi_score: number;
      privacy_score: number;
      coverage_score: number;
      fidelity_score: number;
      utility_score: number;
      linkage_f1: number;
    };
    pii_scrubbed: boolean;
    entity_linkage_preserved: boolean;
    disk_decryption_used: boolean;
  };
  item_count: number;
  data: T[];
  settlement?: {
    chain: PaymentChain;
    tx_hash: string;
    status: string;
    amount_paid: number;
    currency: string;
  };
}
