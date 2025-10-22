use std::{collections::HashMap, str::FromStr};

use futures::{future::join_all, StreamExt};
use reqwest::{ self, header::{HeaderMap, HeaderName, HeaderValue }, Client, ClientBuilder };
use tokio::{fs::{File, OpenOptions}, io::{AsyncSeekExt, AsyncWriteExt}};

pub struct FileInfo {
    pub content_length: u64,
    pub accept_ranges: bool,
    pub file_name: String,
}

pub fn make_default_client(header_hashmap: &HashMap<String, String>) -> Client {
    let mut head_map = HeaderMap::new();
    let client_builder = ClientBuilder::new();
    for (key, val) in header_hashmap {
        let name = HeaderName::from_str(&key).unwrap();
        let value = HeaderValue::from_str(&val).unwrap();

        head_map.insert(name, value);
    }

    client_builder.default_headers(head_map).build().unwrap()
}

pub async fn spawn_download_tasks(client: Client, url: &str, file_info: FileInfo, file_path: &str, chunks: u64) {
    let file = File::create(file_path).await.unwrap();
    file.set_len(file_info.content_length).await.unwrap();
    drop(file);
    let mut handles = Vec::new();
    for i in 0..chunks {

        let client = client.clone();
        let url = url.to_string();
        let file_path = file_path.to_string();
        let content_length = file_info.content_length;

        println!("Spawning task no {}", i);
        let handle = tokio::spawn(async move {
            let start = i * content_length / chunks;
            let end = 
            if i < (chunks - 1) {
                (i + 1) * content_length / chunks - 1
            } else {
                content_length - 1
            };

            let mut file = OpenOptions::new().write(true).open(file_path).await.unwrap();
            file.seek(std::io::SeekFrom::Start(start)).await.unwrap();
            // let part_path = format!("{file_path}.part-{i}");
            file.set_len(end - start + 1).await.unwrap();
            let range = format!("bytes={}-{}", start, end);
            println!("range-{}=>{}-{}", i, start, end);
            let resp = client.get(url).header("Range", range).send().await.unwrap();
            let mut bytes_stream = resp.bytes_stream();

            let mut downloaded_bytes: u64 = 0;
            while let Some(bytes) = bytes_stream.next().await {
                let bytes = bytes.unwrap();
                downloaded_bytes += bytes.len() as u64;
                file.write_all(&bytes).await.unwrap();
            }
            println!("downloaded {downloaded_bytes} for task no {i}");
        });
        println!("Done pawning task no {}", i);
        handles.push(handle);
    }

    println!("Joining all Handles !!");
    join_all(handles).await;
    println!("Done Downloading");
}

pub async fn get_file_info(client: Client, url: &str) -> Result<FileInfo, anyhow::Error> {
    let resp = client.head(url).send().await.unwrap();
    println!("{:?}", resp);
    let content_length: u64 = resp.headers().get("content-length").unwrap().to_str().unwrap().parse().unwrap();
    println!("CONTENT_LENGTH (inside get_file_info): {}",content_length);

    let accept_ranges = match resp.headers().get("accept-ranges") {
        Some(val) => val.to_str().unwrap_or("").eq_ignore_ascii_case("bytes"),
        _ => false,
    };

    let file_name = resp.headers().get("content-disposition").unwrap()
                                        .to_str()
                                        .unwrap().split("filename=").nth(1).unwrap()
                                        .trim_matches('"').trim_matches(';').to_string();

    Ok(FileInfo { content_length, accept_ranges, file_name })
}