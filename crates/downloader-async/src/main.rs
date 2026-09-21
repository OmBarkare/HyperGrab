use crate::downloader::{get_file_info, make_default_client, spawn_download_tasks};
use tokio::sync::mpsc;

mod downloader;
mod server_task;

struct Config {
    workers: usize,
    accept_invalid_certs: bool,
    http2: bool,
}

fn parse_cli_args() -> Config {
    let args: Vec<String> = std::env::args().collect();
    let mut workers = 4;
    let mut accept_invalid_certs = false;
    let mut http2 = false;
    let mut iter = args.iter().skip(1);
    while let Some(arg) = iter.next() {
        if (arg == "--worker" || arg == "-w")
            && let Some(n) = iter.next().and_then(|num| num.parse::<usize>().ok()).filter(|n| *n > 0 && *n <= 64)
        {
            workers = n;
        } else if arg == "--accept-invalid-certs" {
            accept_invalid_certs = true;
        } else if arg == "--http2" || arg == "-h2" {
            http2 = true;
        }
    }
    Config {
        workers,
        accept_invalid_certs,
        http2,
    }
}

#[tokio::main]
async fn main() {
    let config = parse_cli_args();
    println!("Hello async world ! (Workers: {})", config.workers);

    let (tx, mut rx) = mpsc::channel(16);
    let server_handle = tokio::spawn(async move {
        server_task::start_listening("127.0.0.1:7878", tx)
            .await
            .unwrap();
    });
    let downloades_dir = dirs::download_dir().unwrap();
    let downloades_dir = downloades_dir.to_str().unwrap();
    while let Some(res) = rx.recv().await {
        let def_client = make_default_client(&res.headers, &config);
        let file_info = get_file_info(def_client.clone(), &res.url).await.unwrap();
        println!("CONTENT_LENGTH (in main): {}", file_info.content_length);
        let file_path = format!("{}/{}", downloades_dir, file_info.file_name);

        spawn_download_tasks(
            def_client.clone(),
            &res.url,
            file_info,
            &file_path,
            &config,
        )
        .await;
    }

    server_handle.await.unwrap();
}
