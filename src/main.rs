use axum::{routing::get, Json, Router};
use std::net::SocketAddr;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use tyyoj_contracts::SubmissionVerdict;

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let app = Router::new()
        .route("/health", get(health_check))
        .route("/api/v1/verdicts", get(list_verdicts));

    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    tracing::info!("Listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn health_check() -> &'static str {
    "OK"
}

async fn list_verdicts() -> Json<Vec<SubmissionVerdict>> {
    Json(vec![
        SubmissionVerdict::Pending,
        SubmissionVerdict::Compiling,
        SubmissionVerdict::Running,
        SubmissionVerdict::Accepted,
        SubmissionVerdict::WrongAnswer,
        SubmissionVerdict::TimeLimitExceeded,
        SubmissionVerdict::MemoryLimitExceeded,
        SubmissionVerdict::RuntimeError,
        SubmissionVerdict::CompilationError,
        SubmissionVerdict::IdlenessLimitExceeded,
    ])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_health_check() {
        assert_eq!(health_check().await, "OK");
    }
}
