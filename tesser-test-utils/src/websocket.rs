use std::net::SocketAddr;
use std::sync::{Arc, Mutex as StdMutex};

use anyhow::Result;
use futures::{SinkExt, StreamExt};
use serde_json::json;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::oneshot;
use tokio::task::JoinHandle;
use tokio_tungstenite::accept_hdr_async;
use tokio_tungstenite::tungstenite::handshake::server::{Request, Response};
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::WebSocketStream;

use crate::state::{MockExchangeState, PrivateMessage};

pub struct MockWebSocketServer {
    addr: SocketAddr,
    shutdown_tx: Option<oneshot::Sender<()>>,
    handle: JoinHandle<()>,
}

impl MockWebSocketServer {
    pub async fn spawn(state: MockExchangeState) -> Result<Self> {
        let listener = TcpListener::bind(("127.0.0.1", 0)).await?;
        let addr = listener.local_addr()?;
        let (shutdown_tx, mut shutdown_rx) = oneshot::channel();
        let handle = tokio::spawn(async move {
            loop {
                tokio::select! {
                    _ = &mut shutdown_rx => {
                        break;
                    }
                    accept_result = listener.accept() => {
                        match accept_result {
                            Ok((stream, peer)) => {
                                let state = state.clone();
                                tokio::spawn(async move {
                                    if let Err(err) = handle_socket(state, stream, peer).await {
                                        tracing::warn!(error = %err, "websocket connection ended with error");
                                    }
                                });
                            }
                            Err(err) => {
                                tracing::error!(error = %err, "failed to accept websocket connection");
                                break;
                            }
                        }
                    }
                }
            }
        });
        Ok(Self {
            addr,
            shutdown_tx: Some(shutdown_tx),
            handle,
        })
    }

    #[must_use]
    pub fn base_url(&self) -> String {
        format!("ws://{}", self.addr)
    }

    pub async fn shutdown(&mut self) {
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(());
        }
        self.handle.abort();
    }
}

impl Drop for MockWebSocketServer {
    fn drop(&mut self) {
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(());
        }
        self.handle.abort();
    }
}

async fn handle_socket(
    state: MockExchangeState,
    stream: TcpStream,
    _peer: SocketAddr,
) -> Result<()> {
    let captured_path = Arc::new(StdMutex::new(String::new()));
    let path_clone = captured_path.clone();
    let ws_stream = accept_hdr_async(stream, move |req: &Request, resp: Response| {
        if let Ok(mut path) = path_clone.lock() {
            *path = req.uri().path().to_string();
        }
        Ok(resp)
    })
    .await?;
    let path = captured_path
        .lock()
        .map(|guard| guard.clone())
        .unwrap_or_else(|_| "/".to_string());
    if path.starts_with("/v5/public/") {
        handle_public_stream(state, ws_stream, path).await
    } else if path == "/v5/private" {
        handle_private_stream(state, ws_stream).await
    } else {
        tracing::warn!(path = %path, "received websocket connection for unknown path");
        Ok(())
    }
}

async fn handle_public_stream(
    _state: MockExchangeState,
    mut stream: WebSocketStream<TcpStream>,
    topic_path: String,
) -> Result<()> {
    while let Some(msg) = stream.next().await {
        match msg? {
            Message::Text(text) => {
                if let Ok(value) = serde_json::from_str::<serde_json::Value>(&text) {
                    if value.get("op").and_then(|v| v.as_str()) == Some("ping") {
                        let _ = stream
                            .send(Message::Text(
                                json!({"op":"pong","req_id":value.get("req_id")}).to_string(),
                            ))
                            .await;
                        continue;
                    }
                    if value.get("op").and_then(|v| v.as_str()) == Some("subscribe") {
                        let _ = stream
                            .send(Message::Text(json!({"success": true, "conn_id": 0, "req_id": value.get("req_id"), "topic": topic_path}).to_string()))
                            .await;
                    }
                }
            }
            Message::Close(_) => break,
            _ => {}
        }
    }
    Ok(())
}

async fn handle_private_stream(
    state: MockExchangeState,
    stream: WebSocketStream<TcpStream>,
) -> Result<()> {
    let (mut sink, mut source) = stream.split();
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<PrivateMessage>();
    state.set_private_ws_sender(tx.clone()).await;
    let forward = tokio::spawn(async move {
        while let Some(payload) = rx.recv().await {
            if sink.send(Message::Text(payload.to_string())).await.is_err() {
                break;
            }
        }
    });
    while let Some(msg) = source.next().await {
        match msg? {
            Message::Text(text) => {
                if let Ok(value) = serde_json::from_str::<serde_json::Value>(&text) {
                    match value.get("op").and_then(|v| v.as_str()) {
                        Some("ping") => {
                            let _ = tx.send(json!({"op": "pong"}));
                        }
                        Some("auth") => {
                            let _ = tx.send(json!({"op":"auth","success":true}));
                        }
                        _ => {}
                    }
                }
            }
            Message::Close(_) => break,
            _ => {}
        }
    }
    state.clear_private_ws_sender().await;
    forward.abort();
    Ok(())
}
