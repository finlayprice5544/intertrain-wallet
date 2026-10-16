// ─── GET /healthz and /metrics ────────────────────────────────────────────────
// Matches the original rpc-worldstreet.intertrain.online endpoints.

use crate::routes::RouteReg;
use crate::state::AppState;
use axum::extract::State;
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::Router;

/// Simple liveness probe — returns `"ok"` as `text/plain`.
async fn healthz() -> &'static str {
    "ok"
}

/// Prometheus-format metrics.  In simulation mode this emits a minimal
/// gauge set so the wallet's metric polling succeeds.
async fn metrics(State(st): State<AppState>) -> Response {
    let body = format!(
        "# HELP up Whether the server is up.\n\
         # TYPE up gauge\n\
         up 1\n\
         # HELP data_mode Current data mode (1 = active).\n\
         # TYPE data_mode gauge\n\
         data_mode{{mode=\"{:?}\"}} 1\n\
         # HELP rocksdb_ok RocksDB is reachable.\n\
         # TYPE rocksdb_ok gauge\n\
         rocksdb_ok 1\n\
         # HELP clickhouse_ok ClickHouse is connected.\n\
         # TYPE clickhouse_ok gauge\n\
         clickhouse_ok {}\n",
        st.config.data_mode,
        if st.clickhouse.is_some() { 1 } else { 0 },
    );

    (
        [("content-type", "text/plain; version=0.0.4; charset=utf-8")],
        body,
    )
        .into_response()
}

inventory::submit! {
    RouteReg {
        mount: |r: Router<AppState>| r.route("/healthz", get(healthz))
    }
}

inventory::submit! {
    RouteReg {
        mount: |r: Router<AppState>| r.route("/metrics", get(metrics))
    }
}
