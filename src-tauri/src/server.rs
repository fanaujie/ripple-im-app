use crate::app_config::AppConfig;
use crate::ripple_api::RippleApi;
use crate::ripple_syncer::default_event_emitter::DefaultEventEmitter;
use crate::ripple_syncer::incremental_sync_manager::IncrementalSyncManager;
use crate::ripple_syncer::DataSyncManager;
use crate::ripple_ws::ripple_ws_manager::RippleWsManager;
use crate::ripple_ws::sync_aware_ws_message_handler::SyncAwareWsMessageHandler;
use crate::ripple_ws::syncer_control::SyncerControl;
use crate::server::HtmlFile::{AuthFailed, AuthSuccess, AuthSuccessRestart, InvalidState};
use crate::DefaultStoreEngine;
use axum::extract::{Query, State};
use axum::response::Html;
use axum::routing::get;
use axum::Router;
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use tauri::path::BaseDirectory;
use tauri::{AppHandle, Emitter, Manager};
use tokio::net::ToSocketAddrs;
use tokio::task::JoinHandle;

type SyncerAwareMsgHandlerType =
    SyncAwareWsMessageHandler<IncrementalSyncManager<DefaultStoreEngine, DefaultEventEmitter>>;

type WsManagerType = RippleWsManager<SyncerAwareMsgHandlerType>;

enum HtmlFile {
    InvalidState,
    AuthSuccess,
    AuthFailed,
    AuthSuccessRestart,
}

impl Display for HtmlFile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let filename = match self {
            InvalidState => "invalid-state.html",
            AuthSuccess => "auth-success.html",
            AuthFailed => "auth-failed.html",
            AuthSuccessRestart => "auth-success-restart.html",
        };
        write!(f, "{}", filename)
    }
}

async fn load_html_file(app: &AppHandle, html_file: HtmlFile) -> String {
    let path = format!("resources/{}", html_file);
    let resource_path = match app.path().resolve(path, BaseDirectory::Resource) {
        Err(e) => {
            eprintln!("Failed to resolve resource path: {}", e);
            return format!(
                "<h1>Error</h1><p>Failed to resolve resource path: {}</p><p>Details: {}</p>",
                html_file, e
            );
        }
        Ok(path_buffer) => path_buffer,
    };
    match tokio::fs::read_to_string(&resource_path).await {
        Ok(content) => content,
        Err(e) => {
            eprintln!(
                "Failed to read HTML file {} from path {:?}: {}",
                html_file, resource_path, e
            );
            format!(
                "<h1>Error</h1><p>Failed to read file: {}</p><p>Path: {:?}</p><p>Reason: {}</p>",
                html_file, resource_path, e
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
        return Html(load_html_file(&api_state.app_handle, InvalidState).await);
    }
    match ripple.oauth_request_token(params.code).await {
        Ok(_) => {
            // Initialize data first (profile, relations, conversations)
            let app_handle = api_state.app_handle.clone();
            let data_sync = app_handle.state::<DataSyncManager<DefaultStoreEngine>>();
            if let Err(e) = data_sync.init().await {
                eprintln!("Failed to initialize DataSyncManager: {}", e);
                return Html(load_html_file(&api_state.app_handle, AuthSuccessRestart).await);
            }

            // Start WebSocket and syncer in background
            tauri::async_runtime::spawn(async move {
                let ws_manager = app_handle.state::<WsManagerType>();
                let syncer = app_handle.state::<SyncerAwareMsgHandlerType>();
                syncer.start_syncer().await.unwrap();
                let config = app_handle.state::<AppConfig>();
                ws_manager.start(&config.ws_gateway_url).await.unwrap();
            });

            // Emit auth success event to frontend AFTER initialization completes
            if let Err(e) = api_state.app_handle.emit(
                "auth-result",
                AuthenticationState {
                    success: true,
                    message: "Authentication successful".to_string(),
                },
            ) {
                eprintln!("Failed to emit auth-result event: {}", e);
            }

            Html(load_html_file(&api_state.app_handle, AuthSuccess).await)
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
            Html(load_html_file(&api_state.app_handle, AuthFailed).await)
        }
    }
}
