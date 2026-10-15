"""
Storeback Protocol Data Models (RFC-0402)
"""

from dataclasses import dataclass
from typing import Optional, Dict, Any


@dataclass
class X402Challenge:
    version: str
    realm: str
    slug: str
    price_drops: int
    price_xrp: float
    price_usdc_micros: int
    price_usdc: float
    recipient_xrpl: str
    recipient_base_usdc: str
    nonce: str
    expires_at: int
    digest: Optional[str] = None

    @classmethod
    def from_headers(cls, headers: Dict[str, str]) -> "X402Challenge":
        h = {k.lower(): v for k, v in headers.items()}
        drops = int(h.get("x-402-price-drops", 0))
        xrp = float(h.get("x-402-price-xrp", drops / 1_000_000))
        micros = int(h.get("x-402-price-usdc-micros", 0))
        usdc = float(h.get("x-402-price-usdc", micros / 1_000_000))

        return cls(
            version=h.get("x-402-version", "1.0"),
            realm=h.get("www-authenticate", "Storeback").replace('x402 realm="', '').rstrip('"'),
            slug=h.get("x-402-slug", ""),
            price_drops=drops,
            price_xrp=xrp,
            price_usdc_micros=micros,
            price_usdc=usdc,
            recipient_xrpl=h.get("x-402-recipient-xrpl", ""),
            recipient_base_usdc=h.get("x-402-recipient-base-usdc", ""),
            nonce=h.get("x-402-nonce", ""),
            expires_at=int(h.get("x-402-expires", 0)),
            digest=h.get("x-402-digest")
        )


@dataclass
class X402Proof:
    chain: str
    tx_hash: str
    nonce: str
    amount_drops: Optional[int] = None
    amount_usdc_micros: Optional[int] = None
    payer_address: Optional[str] = None

    def to_authorization_header(self) -> str:
        return f"x402 tx_hash={self.tx_hash} nonce={self.nonce} chain={self.chain}"
