"""
Official Storeback Protocol Client (Python)
Negotiates RFC-0402 micropayments over HTTP 402 and XRPL / Base rails.
"""

import json
import base64
from typing import Dict, Any, Optional
import requests

from .models import X402Challenge, X402Proof


class StorebackClient:
    """Client for querying and settling Storeback sovereign knowledge endpoints."""

    def __init__(
        self,
        xrpl_seed: Optional[str] = None,
        base_private_key: Optional[str] = None,
        timeout: int = 15
    ):
        self.xrpl_seed = xrpl_seed
        self.base_private_key = base_private_key
        self.timeout = timeout

    def get_challenge(self, endpoint_url: str) -> X402Challenge:
        """Sends an unauthenticated probe to retrieve the RFC-0402 challenge."""
        resp = requests.get(endpoint_url, timeout=self.timeout)
        if resp.status_code != 402:
            raise ValueError(f"Expected HTTP 402 challenge, received status {resp.status_code}")
        return X402Challenge.from_headers(dict(resp.headers))

    def query_with_proof(
        self,
        endpoint_url: str,
        proof: X402Proof,
        use_x_payment: bool = False
    ) -> Dict[str, Any]:
        """Submits an HTTP query with an on-chain payment proof."""
        headers = {}
        if use_x_payment:
            proof_dict = {
                "chain": proof.chain,
                "tx_hash": proof.tx_hash,
                "nonce": proof.nonce,
            }
            raw_b64 = base64.b64encode(json.dumps(proof_dict).encode("utf-8")).decode("utf-8")
            headers["X-PAYMENT"] = raw_b64
        else:
            headers["Authorization"] = proof.to_authorization_header()

        resp = requests.get(endpoint_url, headers=headers, timeout=self.timeout)
        if resp.status_code == 200:
            return resp.json()
        elif resp.status_code == 402:
            body = resp.json() if resp.headers.get("content-type", "").startswith("application/json") else {}
            err = body.get("verification_error", "Payment rejected")
            raise PermissionError(f"HTTP 402 Payment Verification Failed: {err}")
        else:
            resp.raise_for_status()
            return {}
