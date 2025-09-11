use crate::ripple::RippleApi;
use crate::store_engine::{MemoryStore, StoreEngine};
use crate::DefaultStoreEngine;
use axum::extract::{Query, State};
use axum::response::Html;
use axum::routing::get;
use axum::Router;
use serde::{Deserialize, Serialize};
use tauri::path::BaseDirectory;
use tauri::{AppHandle, Emitter, Manager};
use tokio::net::ToSocketAddrs;
use tokio::task::JoinHandle;

async fn load_html_file(app: &AppHandle, filename: &str) -> String {
    let path = format!("resources/{}", filename);
    let resource_path = match app.path().resolve(path, BaseDirectory::Resource) {
        Err(e) => {
            eprintln!("Failed to resolve resource path: {}", e);
            return format!(
                "<h1>Error</h1><p>Failed to resolve resource path: {}</p><p>Details: {}</p>",
                filename, e
            );
        }
        Ok(path_buffer) => path_buffer,
    };
    match tokio::fs::read_to_string(&resource_path).await {
        Ok(content) => content,
        Err(e) => {
            eprintln!(
                "Failed to read HTML file {} from path {:?}: {}",
                filename, resource_path, e
            );
            format!(
                "<h1>Error</h1><p>Failed to read file: {}</p><p>Path: {:?}</p><p>Reason: {}</p>",
                filename, resource_path, e
            )
        }
    }
}

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
) -> Html<String> {
    let ripple = api_state
        .app_handle
        .state::<RippleApi<DefaultStoreEngine>>();
    if !ripple.oauth_state_equal(&params.state) {
        return Html(load_html_file(&api_state.app_handle, "invalid-state.html").await);
    }
    match ripple.oauth_request_token(params.code).await {
        Ok(_) => {
            // Emit auth success event to frontend
            if let Err(e) = api_state.app_handle.emit(
                "auth-result",
                AuthenticationState {
                    success: true,
                    message: "Authentication successful".to_string(),
                },
            ) {
                eprintln!("Failed to emit auth-result event: {}", e);
                return Html(
                    load_html_file(&api_state.app_handle, "auth-success-restart.html").await,
                );
            }
            Html(load_html_file(&api_state.app_handle, "auth-success.html").await)
        }
        Err(e) => {
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
            Html(load_html_file(&api_state.app_handle, "auth-failed.html").await)
        }
    }
}
