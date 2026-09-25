use std::net::SocketAddr;

use axum::{routing::get, Json, Router};
use serde::Serialize;

#[derive(Serialize)]
struct Status {
    ok: bool,
    service: &'static str,
    framework: &'static str,
}

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/", get(status))
        .route("/health", get(status));

    let port = std::env::var("PORT")
        .ok()
        .and_then(|value| value.parse::<u16>().ok())
        .unwrap_or(8080);
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .unwrap_or_else(|err| panic!("failed to bind {addr}: {err}"));

    println!("listening on http://{addr}");
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap_or_else(|err| panic!("server error: {err}"));
}

async fn status() -> Json<Status> {
    Json(Status {
        ok: true,
        service: "test-server",
        framework: "axum",
    })
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        () = ctrl_c => {}
        () = terminate => {}
    }
}
