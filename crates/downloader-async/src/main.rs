use crate::downloader::{get_file_info, make_default_client, spawn_download_tasks};
use tokio::sync::mpsc;

mod downloader;
mod server_task;
pub mod tls;

pub struct Config {
    pub workers: usize,
    pub accept_invalid_certs: bool,
    pub http2: bool,
    pub url: Option<String>,
}

fn print_help() {
    println!(
r#"HyperGrab - High-performance concurrent HTTP download manager

USAGE:
    downloader-async [OPTIONS]

OPTIONS:
    -w, --workers <num_workers>        Number of concurrent download workers (default: 4, range: 1-64)
        --url <url>            Directly download from URL without waiting for browser extension
        --http2, -h2           Enable HTTP/2 protocol negotiation (default: HTTP/1.1)
        --accept-invalid-certs Disable TLS certificate verification (for local/self-signed testing)
    -h, --help                 Print this help information
"#
    );
}

fn parse_cli_args() -> Config {
    let args: Vec<String> = std::env::args().collect();
    let mut workers = 4;
    let mut accept_invalid_certs = false;
    let mut http2 = false;
    let mut url = None;
    let mut iter = args.iter().skip(1);
    while let Some(arg) = iter.next() {
        if arg == "-h" || arg == "--help" {
            print_help();
            std::process::exit(0);
        } else if (arg == "--worker" || arg == "--workers" || arg == "-w")
            && let Some(n) = iter.next().and_then(|num| num.parse::<usize>().ok()).filter(|n| *n > 0 && *n <= 64)
        {
            workers = n;
        } else if arg == "--accept-invalid-certs" {
            accept_invalid_certs = true;
        } else if arg == "--http2" || arg == "-h2" {
            http2 = true;
        } else if arg == "--url" && let Some(u) = iter.next() {
            url = Some(u.clone());
        }
    }
    Config {
        workers,
        accept_invalid_certs,
        http2,
        url,
    }
}

#[tokio::main]
async fn main() {
    let config = parse_cli_args();
    println!("Hello async world ! (Workers: {})", config.workers);

    let downloades_dir = dirs::download_dir().unwrap();
    let downloades_dir = downloades_dir.to_str().unwrap();

    if let Some(ref url) = config.url {
        let empty_headers = std::collections::HashMap::new();
        let def_client = make_default_client(&empty_headers, &config);
        let file_info = get_file_info(def_client.clone(), url).await.unwrap();
        println!("CONTENT_LENGTH (in main): {}", file_info.content_length);
        let file_path = format!("{}/{}", downloades_dir, file_info.file_name);

        spawn_download_tasks(
            def_client,
            url,
            file_info,
            &file_path,
            &config,
        )
        .await;
    } else {
        println!("Server spawning on 127.0.0.1:7878...");
        let (tx, mut rx) = mpsc::channel(16);
        let server_handle = tokio::spawn(async move {
            server_task::start_listening("127.0.0.1:7878", tx)
                .await
                .unwrap();
        });
        println!("Server spawned");

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
}
