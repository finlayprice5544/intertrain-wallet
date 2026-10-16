// Bridge lane status and simulated deposit/withdrawal tracking.
// The wallet connects to Ethereum (Sepolia) and Solana (Devnet) bridge lanes.
// All state here is synthetic — no real cross-chain messages are sent.
use serde_json::{json, Value};

/// Static bridge lane status — always reports connected.
pub fn status() -> Value {
    json!({
        "ethereum": {
            "network": "sepolia",
            "connected": true,
            "contract": "0x0000000000000000000000000000000000000000",
            "confirmations_required": 12
        },
        "solana": {
            "network": "devnet",
            "mode": "light",
            "connected": true,
            "program_id": "11111111111111111111111111111111",
            "confirmations_required": 32
        }
    })
}

/// Simulated bridge deposit event.
pub fn simulated_deposit(lane: &str, amount: &str, asset: &str) -> Value {
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    json!({
        "lane": lane,
        "direction": "deposit",
        "external_tx": format!("0x{:064x}", ts),
        "amount": amount,
        "asset": asset,
        "status": "confirmed",
        "timestamp": ts
    })
}

/// Simulated bridge withdrawal event.
pub fn simulated_withdrawal(lane: &str, amount: &str, asset: &str) -> Value {
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    json!({
        "lane": lane,
        "direction": "withdrawal",
        "external_tx": format!("0x{:064x}", ts.wrapping_add(1)),
        "amount": amount,
        "asset": asset,
        "status": "pending",
        "timestamp": ts
    })
}
