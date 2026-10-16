// ─── EXAMPLE ROUTE — copy this pattern per captured endpoint ─────────────────
// Shows: respect DATA_MODE, read seed from RocksDB, optionally enrich live via a
// service, return the SAME JSON envelope as the captured response. Rename to
// coins.rs (and add `pub mod coins;` in mod.rs) if /api/coins is real; else delete.
use crate::config::DataMode;
use crate::routes::RouteReg;
use crate::state::AppState;
use crate::{db::rocks, services::dexscreener};
use axum::{extract::State, routing::get, Json, Router};
use serde_json::{json, Value};

async fn coins(State(st): State<AppState>) -> Json<Value> {
    // 1) SEED from RocksDB (always available offline)
    let seeded = rocks::seed_all(&st.rocks, "coins");
    let mut rows = match seeded {
        Value::Array(a) => a,
        _ => vec![],
    };

    // 2) LIVE enrich (unless DATA_MODE=seed)
    if st.config.data_mode != DataMode::Seed {
        for row in rows.iter_mut() {
            if let Some(mint) = row.get("mint").and_then(|m| m.as_str()) {
                let price = dexscreener::price_usd(&st.config, &st.http, mint).await;
                if price > 0.0 {
                    row["priceUsd"] = json!(price);
                }
            }
        }
    }

    // Match the captured envelope exactly (here: { "coins": [...] }).
    Json(json!({ "coins": rows }))
}

inventory::submit! { RouteReg { mount: |r: Router<AppState>| r.route("/api/coins", get(coins)) } }
