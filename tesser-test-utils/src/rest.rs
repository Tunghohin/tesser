use std::convert::Infallible;
use std::net::SocketAddr;

use anyhow::Result;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Method, Request, Response, Server, StatusCode};
use serde_json::json;
use tokio::net::TcpListener;
use tokio::sync::oneshot;
use tokio::task::JoinHandle;

use crate::state::MockExchangeState;

pub struct MockRestApi {
    addr: SocketAddr,
    shutdown_tx: Option<oneshot::Sender<()>>,
    handle: JoinHandle<()>,
}

impl MockRestApi {
    pub async fn spawn(state: MockExchangeState) -> Result<Self> {
        let listener = TcpListener::bind(("127.0.0.1", 0)).await?;
        let addr = listener.local_addr()?;
        let std_listener = listener.into_std()?;
        std_listener.set_nonblocking(true)?;
        let (shutdown_tx, shutdown_rx) = oneshot::channel();
        let make_svc = make_service_fn(move |_| {
            let state = state.clone();
            async move {
                Ok::<_, Infallible>(service_fn(move |req| {
                    let state = state.clone();
                    async move { Ok::<_, Infallible>(route(req, state).await) }
                }))
            }
        });
        let server = Server::from_tcp(std_listener)?.serve(make_svc);
        let handle = tokio::spawn(async move {
            if let Err(err) = server
                .with_graceful_shutdown(async {
                    let _ = shutdown_rx.await;
                })
                .await
            {
                tracing::error!(error = %err, "mock REST server exited with error");
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
        format!("http://{}", self.addr)
    }

    pub async fn shutdown(&mut self) {
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(());
        }
        self.handle.abort();
    }
}

impl Drop for MockRestApi {
    fn drop(&mut self) {
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(());
        }
        self.handle.abort();
    }
}

async fn route(req: Request<Body>, state: MockExchangeState) -> Response<Body> {
    let _ = state;
    match (req.method(), req.uri().path()) {
        (&Method::POST, "/v5/order/create") => placeholder_response("order/create"),
        (&Method::POST, "/v5/order/cancel") => placeholder_response("order/cancel"),
        (&Method::GET, "/v5/position/list") => placeholder_response("position/list"),
        (&Method::GET, "/v5/account/wallet-balance") => {
            placeholder_response("account/wallet-balance")
        }
        (&Method::GET, "/v5/execution/list") => placeholder_response("execution/list"),
        _ => not_found(),
    }
}

fn placeholder_response(endpoint: &str) -> Response<Body> {
    json_response(
        StatusCode::NOT_IMPLEMENTED,
        json!({
            "retCode": -1,
            "retMsg": format!("{endpoint} not implemented"),
            "result": serde_json::Value::Null,
            "retExtInfo": serde_json::Value::Null,
            "time": 0,
        }),
    )
}

fn not_found() -> Response<Body> {
    json_response(
        StatusCode::NOT_FOUND,
        json!({
            "retCode": 404,
            "retMsg": "endpoint not found",
            "result": serde_json::Value::Null,
            "retExtInfo": serde_json::Value::Null,
            "time": 0,
        }),
    )
}

fn json_response(status: StatusCode, body: serde_json::Value) -> Response<Body> {
    Response::builder()
        .status(status)
        .header("Content-Type", "application/json")
        .body(Body::from(body.to_string()))
        .unwrap()
}
