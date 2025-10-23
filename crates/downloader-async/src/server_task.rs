
use std::{collections::HashMap, sync::Arc};
use axum::{Router, routing::post, Json, extract::State, http::StatusCode};
use serde::Deserialize;
use anyhow::Error;
use tokio::sync::mpsc;
use tokio::net::TcpListener;

/// struct to store the information sent to our localhost
/// by our extension. Stores the download url as String
/// and the headers as HashMap
#[derive(Deserialize, Debug, Clone)]
pub struct DownloadRequest {
    pub url: String,
    pub headers: HashMap<String, String>,   
}

/// A mpsc::Sender struct which we will wrap in arc so that we can 
/// share this with the handler function, which cannot have arguments
/// other than extractors
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
    Json(download_request): Json<DownloadRequest>
) -> StatusCode {
    
    println!("recieved url: {} \n headers: {:?}", download_request.url, download_request.headers);
    state.tx.send(download_request).await.unwrap();
    StatusCode::OK
}
