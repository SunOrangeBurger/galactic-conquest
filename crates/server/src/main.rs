//! Galactic Conquest match server — Phase 0.
//!
//! This is a starting skeleton, not the full tick orchestrator. It proves
//! the wire path (axum + contract structs) end-to-end: an agent can POST
//! its actions and get validated back. The real bounded-synchronous
//! multi-agent tick loop (Section 2.3 — collect all agents' actions within
//! `deadline_ms`, resolve simultaneously at year-end) still needs a match
//! coordinator wired around `engine::state::GalaxyState`, which is left as
//! the next build step (Section 12, step 2).

use axum::{
    extract::State,
    routing::{get, post},
    Json, Router,
};
use contract::AgentResponse;
use sim::state::GalaxyState;
use sim::validation::validate_actions;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Clone)]
struct AppState {
    galaxy: Arc<Mutex<GalaxyState>>,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let state = AppState {
        galaxy: Arc::new(Mutex::new(GalaxyState::default())),
    };

    let app = Router::new()
        .route("/health", get(health))
        .route("/tick/:civ_id/submit", post(submit_actions))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080")
        .await
        .expect("failed to bind port 8080");

    tracing::info!("server listening on 0.0.0.0:8080");
    axum::serve(listener, app).await.expect("server error");
}

async fn health() -> &'static str {
    "ok"
}

/// Per Section 3.3: malformed JSON is rejected as a full no-op for the tick
/// (handled by axum's Json extractor failing before this handler even
/// runs). Individual invalid actions inside a well-formed body are rejected
/// one at a time, tagged with a reason, rather than failing the whole
/// submission.
async fn submit_actions(
    State(state): State<AppState>,
    axum::extract::Path(civ_id): axum::extract::Path<String>,
    Json(response): Json<AgentResponse>,
) -> Json<Vec<sim::validation::ActionValidationError>> {
    let galaxy = state.galaxy.lock().await;
    let (_valid, errors) = validate_actions(&galaxy, &civ_id, &response.actions);
    Json(errors)
}
