use tokio::sync::mpsc;

mod server_task;
mod downloader;

#[tokio::main]
async fn main() {
    println!("Hello async world !");
    
    let (tx, mut rx) = mpsc::channel(16);
    let server_handle = tokio::spawn(async move {
        server_task::start_listening("127.0.0.1:7878", tx).await.unwrap();
    });

    server_handle.await.unwrap();
}