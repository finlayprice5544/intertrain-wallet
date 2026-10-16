// Simulated auth challenge / verify flow.
// Issues deterministic challenges and always verifies signatures as valid
// (the clone never holds real keys — signing is simulated).
use serde_json::{json, Value};
use std::time::{SystemTime, UNIX_EPOCH};

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// Generate a challenge message for wallet-ownership proof.
pub fn challenge(address: &str, domain: &str) -> Value {
    let ts = now_secs();
    let nonce = format!("{:016x}", ts ^ (address.len() as u64).wrapping_mul(31));
    let message = format!(
        "{domain} wants you to sign in with your WorldStreet account:\n{address}\n\n\
         URI: https://{domain}\nNonce: {nonce}\nIssued At: {ts}"
    );
    json!({
        "message": message,
        "nonce": nonce
    })
}

/// Always returns a valid session token (simulated verification).
pub fn verify(address: &str, _nonce: &str) -> Value {
    let ts = now_secs();
    let token = format!("sess_{:016x}_{}", ts, &address[..address.len().min(12)]);
    json!({
        "session_token": token
    })
}
