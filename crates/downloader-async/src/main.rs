use crate::downloader::{get_file_info, make_default_client, spawn_download_tasks};
use dirs;
use tokio::sync::mpsc;

mod downloader;
mod server_task;

fn parse_num_workers() -> usize {
    let args: Vec<String> = std::env::args().collect();
    let mut iter = args.iter().skip(1);
    while let Some(arg) = iter.next() {
        if arg == "-w" || arg == "--workers" {
            if let Some(val) = iter.next() {
                if let Ok(n) = val.parse::<usize>() {
                    if n > 0 {
                        return n;
                    }
                }
            }
        }
    }
    return 4
}

#[tokio::main]
async fn main() {
    let num_workers = parse_num_workers();
    println!("Hello async world ! (Workers: {num_workers})");

    let (tx, mut rx) = mpsc::channel(16);
    let server_handle = tokio::spawn(async move {
        server_task::start_listening("127.0.0.1:7878", tx)
            .await
            .unwrap();
    });
    let downloades_dir = dirs::download_dir().unwrap();
    let downloades_dir = downloades_dir.to_str().unwrap();
    while let Some(res) = rx.recv().await {
        let def_client = make_default_client(&res.headers);
        let file_info = get_file_info(def_client.clone(), &res.url).await.unwrap();
        println!("CONTENT_LENGTH (in main): {}", file_info.content_length);
        let file_path = format!("{}/{}", downloades_dir, file_info.file_name);

        spawn_download_tasks(
            def_client.clone(),
            &res.url,
            file_info,
            &file_path,
            num_workers,
        )
        .await;
    }

    server_handle.await.unwrap();
}
