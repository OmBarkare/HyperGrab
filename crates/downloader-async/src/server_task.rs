
use std::{collections::HashMap, sync::Arc};
use axum::{Router, routing::post, Json, extract::State, http::StatusCode};
use serde::Deserialize;
use anyhow::Error;
use tokio::sync::mpsc;
use tokio::net::TcpListener;


#[derive(Deserialize, Debug, Clone)]
pub struct DownloadRequest {
    pub url: String,
    pub headers: HashMap<String, String>,   
}

pub struct AppState {
    tx: mpsc::Sender<DownloadRequest>,
}
pub async fn start_listening(addr: &str, tx: mpsc::Sender<DownloadRequest>) -> Result<(), Error> {
    let state = Arc::new(AppState{tx});
    let app = Router::new().route("/", post(handler)).with_state(state);
    let listener = TcpListener::bind(addr).await?;

    axum::serve(listener, app).await?;
    Ok(())
}

async fn handler(
    State(state): State<Arc<AppState>>,
    Json(downloadRequest): Json<DownloadRequest>
) -> StatusCode {
    
    println!("recieved url: {} \n headers: {:?}", downloadRequest.url, downloadRequest.headers);
    state.tx.send(downloadRequest).await.unwrap();
    StatusCode::OK
}
