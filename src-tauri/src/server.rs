use crate::oauth_client::OauthClient;
use axum::extract::{Query, State};
use axum::response::Html;
use axum::routing::get;
use axum::Router;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::{AppHandle, Emitter, Manager};
use tokio::net::ToSocketAddrs;
use tokio::task::JoinHandle;

pub struct Server {
    close_tx: Option<tokio::sync::oneshot::Sender<()>>,
    server_handle: Option<JoinHandle<()>>,
}

impl Server {
    pub fn new() -> Self {
        Server {
            close_tx: None,
            server_handle: None,
        }
    }

    pub async fn start(&mut self, addr: impl ToSocketAddrs, app_handle: AppHandle) {
        if self.server_handle.is_some() {
            println!("Server is already running.");
            return;
        }

        let app = Router::new()
            .route("/callback", get(handler))
            .with_state(ApiState { app_handle });
        let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
        println!("listening on {}", listener.local_addr().unwrap());
        let (close_tx, close_rx) = tokio::sync::oneshot::channel::<()>();
        self.close_tx = Some(close_tx);

        let handle = tokio::spawn(async move {
            axum::serve(listener, app)
                .with_graceful_shutdown(async move {
                    _ = close_rx.await;
                    println!("Server is shutting down gracefully...");
                })
                .await
                .unwrap();
        });

        self.server_handle = Some(handle);
    }

    pub async fn stop(&mut self) {
        println!("Stopping server...");
        if let Some(tx) = self.close_tx.take() {
            let _ = tx.send(());
            println!("Server stop signal sent.");

            if let Some(handle) = self.server_handle.take() {
                let _ = handle.await;
                println!("Server stopped successfully.");
            }
        } else {
            println!("Server is not running.");
        }
    }
}

#[derive(Clone, Serialize)]
struct AuthenticationState {
    success: bool,
    message: String,
}

#[derive(Deserialize)]
struct CallbackParams {
    code: String,
    state: String,
}

#[derive(Clone)]
struct ApiState {
    app_handle: AppHandle,
}

async fn handler(
    State(api_state): State<ApiState>,
    Query(params): Query<CallbackParams>,
) -> Html<&'static str> {
    let client_state = api_state
        .app_handle
        .state::<Arc<tokio::sync::Mutex<OauthClient>>>();
    let client = client_state.lock().await;
    if !client.state_equal(&params.state) {
        return Html("<h1>Invalid state parameter</h1>");
    }
    match client.request_token(params.code).await {
        Ok(token) => {
            println!("Token received: {}", token);

            // Emit auth success event to frontend
            if let Err(e) = api_state.app_handle.emit(
                "auth-result",
                AuthenticationState {
                    success: true,
                    message: "Authentication successful".to_string(),
                },
            ) {
                eprintln!("Failed to emit auth-result event: {}", e);
                return Html("<h1>Authentication successful, but failed to notify frontend</h1>");
            }
            Html("<h1>Authentication successful! You can close this window.</h1>")
        }
        Err(e) => {
            println!("Error requesting token: {}", e);
            // Emit auth failure event to frontend
            if let Err(e) = api_state.app_handle.emit(
                "auth-result",
                AuthenticationState {
                    success: false,
                    message: e.to_string(),
                },
            ) {
                eprintln!("Failed to emit auth-result event: {}", e);
            }
            Html("<h1>Authentication failed</h1>")
        }
    }
}
