use tokio::sync::mpsc;
use crate::{downloader::{get_file_info, make_default_client, spawn_download_tasks}};
use dirs;

mod server_task;
mod downloader;

#[tokio::main]
async fn main() {
    println!("Hello async world !");
    const CHUNKS: u64 = 4;

    let (tx, mut rx) = mpsc::channel(16);
    let server_handle = tokio::spawn(async move {
        server_task::start_listening("127.0.0.1:7878", tx).await.unwrap();
    });
    let downloades_dir = dirs::download_dir().unwrap();
    let downloades_dir = downloades_dir.to_str().unwrap();
    while let Some(res) = rx.recv().await {
        let def_client = make_default_client(&res.headers);
        let file_info = get_file_info(def_client.clone(), &res.url).await.unwrap();
        println!("CONTENT_LENGTH (in main): {}",file_info.content_length);
        let file_path = format!("{}/{}", downloades_dir, file_info.file_name);

        spawn_download_tasks(def_client.clone(), &res.url, file_info, &file_path, CHUNKS).await;
    }

    server_handle.await.unwrap();
}