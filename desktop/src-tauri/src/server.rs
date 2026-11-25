use crate::models::{ClientCommand, ConnectionInfo, ServerEvent};
use crate::persist;
use crate::state::{Core, Providers};
use anyhow::{bail, Context, Result};
use axum::extract::ws::{CloseFrame, Message, WebSocket, WebSocketUpgrade};
use axum::extract::{ConnectInfo, Query, State};
use axum::http::{header, HeaderMap, HeaderValue, Method, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{Json, Router};
use futures_util::{SinkExt, StreamExt};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::net::{IpAddr, SocketAddr};
use std::path::PathBuf;
use std::sync::mpsc;
use std::sync::Arc;
use std::time::Duration;
use subtle::ConstantTimeEq;
use tokio::net::TcpListener;
use tokio::sync::broadcast;
use tower_http::cors::{AllowOrigin, CorsLayer};
use tracing::{info, warn};

const ALLOWED_ORIGINS: [&str; 5] = [
    "http://localhost:1420",
    "http://127.0.0.1:1420",
    "tauri://localhost",
    "http://tauri.localhost",
    "https://tauri.localhost",
];

pub fn spawn_server() -> Result<ConnectionInfo> {
    load_env();

    let data_dir = persist::data_dir();
    std::fs::create_dir_all(&data_dir)
        .with_context(|| format!("No se pudo crear {}", data_dir.display()))?;
    let token = persist::load_or_create_token(&data_dir);
    let bind = std::env::var("GREEK_BIND").unwrap_or_else(|_| "127.0.0.1:4319".to_string());

    let (ready_tx, ready_rx) = mpsc::channel::<std::result::Result<ConnectionInfo, String>>();

    std::thread::Builder::new()
        .name("greek-core".into())
        .spawn(move || {
            let runtime = match tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
            {
                Ok(runtime) => runtime,
                Err(error) => {
                    let _ = ready_tx.send(Err(format!("No se pudo crear el runtime: {error}")));
                    return;
                }
            };

            runtime.block_on(async move {
                match start_async(token, bind, data_dir).await {
                    Ok((connection, core, listener)) => {
                        let _ = ready_tx.send(Ok(connection.clone()));
                        info!("Greek escucha en {} (LAN {})", connection.url, connection.lan_url);
                        let app = router(core);
                        if let Err(error) = axum::serve(
                            listener,
                            app.into_make_service_with_connect_info::<SocketAddr>(),
                        )
                        .await
                        {
                            warn!("El servidor se detuvo: {error}");
                        }
                    }
                    Err(error) => {
                        let _ = ready_tx.send(Err(format!("{error:#}")));
                    }
                }
            });
        })
        .context("No se pudo iniciar el hilo del núcleo.")?;

    match ready_rx.recv_timeout(Duration::from_secs(20)) {
        Ok(Ok(info)) => Ok(info),
        Ok(Err(error)) => bail!("{error}"),
        Err(_) => bail!("El núcleo tardó demasiado en iniciar."),
    }
}

fn load_env() {
    let _ = dotenvy::dotenv();
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            let _ = dotenvy::from_path(dir.join(".env"));
        }
    }
}

async fn start_async(
    token: String,
    bind: String,
    data_dir: PathBuf,
) -> Result<(ConnectionInfo, Arc<Core>, TcpListener)> {
    let requested: SocketAddr = bind
        .parse()
        .with_context(|| format!("GREEK_BIND no es una dirección válida: {bind}"))?;
    let (listener, address) = bind_listener(requested).await?;

    let url = format!("http://127.0.0.1:{}", address.port());
    let lan_url = local_ipv4()
        .map(|ip| format!("http://{ip}:{}", address.port()))
        .unwrap_or_else(|| url.clone());

    let info = ConnectionInfo {
        url,
        lan_url,
        token: token.clone(),
        port: address.port(),
        bind: address.to_string(),
    };

    let http = reqwest::Client::builder()
        .user_agent(format!("Greek/{}", crate::VERSION))
        .timeout(Duration::from_secs(30))
        .build()
        .context("No se pudo crear el cliente HTTP.")?;

    let providers = providers_from_env(&http).await;
    let core = Core::new(token, info.clone(), data_dir, providers, http);
    Ok((info, core, listener))
}

async fn bind_listener(requested: SocketAddr) -> Result<(TcpListener, SocketAddr)> {
    let mut last_error = None;
    for offset in 0..10u16 {
        let mut candidate = requested;
        if requested.port() != 0 {
            candidate.set_port(requested.port().saturating_add(offset));
        }
        match TcpListener::bind(candidate).await {
            Ok(listener) => {
                let address = listener.local_addr()?;
                return Ok((listener, address));
            }
            Err(error) => last_error = Some(error),
        }
    }

    bail!(
        "No se pudo abrir el puerto {}: {}",
        requested.port(),
        last_error
            .map(|error| error.to_string())
            .unwrap_or_else(|| "sin detalle".to_string())
    )
}

fn local_ipv4() -> Option<IpAddr> {
    let socket = std::net::UdpSocket::bind("0.0.0.0:0").ok()?;
    socket.connect("8.8.8.8:80").ok()?;
    let address = socket.local_addr().ok()?;
    let ip = address.ip();
    if ip.is_loopback() || ip.is_unspecified() {
        None
    } else {
        Some(ip)
    }
}

fn env_nonempty(name: &str) -> Option<String> {
    std::env::var(name)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

async fn providers_from_env(http: &reqwest::Client) -> Providers {
    let configured_ollama = env_nonempty("GREEK_OLLAMA_MODEL");
    let mut providers = Providers {
        openrouter_key: env_nonempty("OPENROUTER_API_KEY"),
        openrouter_model: env_nonempty("GREEK_OPENROUTER_MODEL"),
        ollama_model: configured_ollama.clone(),
        ollama_available: false,
    };

    let probe = http
        .get("http://127.0.0.1:11434/api/tags")
        .timeout(Duration::from_millis(1_500))
        .send()
        .await;

    if let Ok(response) = probe {
        if response.status().is_success() {
            if let Ok(body) = response.json::<Value>().await {
                let names: Vec<String> = body
                    .get("models")
                    .and_then(Value::as_array)
                    .map(|models| {
                        models
                            .iter()
                            .filter_map(|model| model.get("name").and_then(Value::as_str))
                            .map(str::to_string)
                            .collect()
                    })
                    .unwrap_or_default();

                let configured_present = configured_ollama
                    .as_ref()
                    .map(|configured| {
                        names.iter().any(|name| {
                            name == configured || name.starts_with(&format!("{configured}:"))
                        })
                    })
                    .unwrap_or(true);

                if configured_ollama.is_some() {
                    providers.ollama_available = configured_present;
                } else if let Some(first) = names.first() {
                    providers.ollama_model = Some(first.clone());
                    providers.ollama_available = true;
                }
            }
        }
    }

    providers
}

fn cors_layer() -> CorsLayer {
    CorsLayer::new()
        .allow_origin(AllowOrigin::list(
            ALLOWED_ORIGINS
                .iter()
                .map(|origin| HeaderValue::from_static(origin)),
        ))
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        .allow_headers(tower_http::cors::Any)
}

fn router(core: Arc<Core>) -> Router {
    Router::new()
        .route("/api/health", get(health))
        .route("/api/dev/bootstrap", get(bootstrap))
        .route("/ws", get(ws_handler))
        .layer(cors_layer())
        .with_state(core)
}

async fn health() -> Json<Value> {
    Json(json!({ "status": "ok", "version": crate::VERSION }))
}

async fn bootstrap(
    State(core): State<Arc<Core>>,
    ConnectInfo(address): ConnectInfo<SocketAddr>,
) -> Response {
    if !address.ip().is_loopback() {
        return (
            StatusCode::FORBIDDEN,
            Json(json!({ "error": "El bootstrap solo está disponible en loopback." })),
        )
            .into_response();
    }
    Json(core.info.clone()).into_response()
}

fn constant_time_eq(provided: &str, expected: &str) -> bool {
    if provided.len() != expected.len() {
        return false;
    }
    provided.as_bytes().ct_eq(expected.as_bytes()).into()
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    State(core): State<Arc<Core>>,
    Query(query): Query<HashMap<String, String>>,
    ConnectInfo(address): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
) -> Response {
    if let Some(origin) = headers.get(header::ORIGIN).and_then(|value| value.to_str().ok()) {
        if !ALLOWED_ORIGINS.contains(&origin) {
            warn!("Conexión WebSocket rechazada por origen desde {address}");
            return (StatusCode::FORBIDDEN, "origen no permitido").into_response();
        }
    }

    let provided = query.get("token").cloned().unwrap_or_default();
    if !constant_time_eq(&provided, &core.token) {
        warn!("Conexión WebSocket rechazada por token desde {address}");
        return ws
            .on_upgrade(|mut socket| async move {
                let _ = socket
                    .send(Message::Close(Some(CloseFrame {
                        code: 4401,
                        reason: "token inválido".into(),
                    })))
                    .await;
            })
            .into_response();
    }

    ws.max_message_size(64 * 1024)
        .on_upgrade(move |socket| client_loop(socket, core))
        .into_response()
}

async fn client_loop(socket: WebSocket, core: Arc<Core>) {
    let (mut sender, mut receiver) = socket.split();
    let mut events = core.tx.subscribe();

    let initial = ServerEvent::Snapshot {
        snapshot: core.snapshot(),
    };
    if let Ok(json) = serde_json::to_string(&initial) {
        if sender.send(Message::Text(json.into())).await.is_err() {
            return;
        }
    }

    loop {
        tokio::select! {
            incoming = receiver.next() => {
                match incoming {
                    Some(Ok(Message::Text(text))) => {
                        match serde_json::from_str::<ClientCommand>(text.as_str()) {
                            Ok(ClientCommand::Ping) => {
                                if sender.send(Message::Text("{\"type\":\"pong\"}".into())).await.is_err() {
                                    break;
                                }
                            }
                            Ok(command) => {
                                if let Some(message) = core.handle_command(command).await {
                                    let event = ServerEvent::Error { message };
                                    if let Ok(json) = serde_json::to_string(&event) {
                                        if sender.send(Message::Text(json.into())).await.is_err() {
                                            break;
                                        }
                                    }
                                }
                            }
                            Err(_) => {
                                let event = ServerEvent::Error {
                                    message: "Comando no reconocido.".to_string(),
                                };
                                if let Ok(json) = serde_json::to_string(&event) {
                                    if sender.send(Message::Text(json.into())).await.is_err() {
                                        break;
                                    }
                                }
                            }
                        }
                    }
                    Some(Ok(Message::Close(_))) | None => break,
                    Some(Ok(_)) => {}
                    Some(Err(_)) => break,
                }
            }
            event = events.recv() => {
                match event {
                    Ok(event) => {
                        if let Ok(json) = serde_json::to_string(&event) {
                            if sender.send(Message::Text(json.into())).await.is_err() {
                                break;
                            }
                        }
                    }
                    Err(broadcast::error::RecvError::Lagged(_)) => {
                        let event = ServerEvent::Snapshot {
                            snapshot: core.snapshot(),
                        };
                        if let Ok(json) = serde_json::to_string(&event) {
                            if sender.send(Message::Text(json.into())).await.is_err() {
                                break;
                            }
                        }
                    }
                    Err(broadcast::error::RecvError::Closed) => break,
                }
            }
        }
    }

    let _ = sender.close().await;
}
