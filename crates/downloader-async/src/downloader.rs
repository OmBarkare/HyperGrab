use std::{collections::HashMap, fmt::format, str::FromStr};

use futures::StreamExt;
use reqwest::{ self, header::{ HeaderMap, HeaderName, HeaderValue }, Client, ClientBuilder, Response };
use tokio::{fs::{File, OpenOptions}, io::{AsyncSeekExt, AsyncWriteExt}};

use crate::server_task::DownloadRequest;

struct FileInfo {
    content_length: u64,
    accept_ranges: bool,
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

pub async fn get_file_info(client: Client, url: &str) -> Result<FileInfo, anyhow::Error> {
    let resp = client.head(url).send().await.unwrap();
    println!("{:?}", resp);
    let content_length = resp.content_length().unwrap();

    let accept_ranges = match resp.headers().get("accept-ranges") {
        Some(val) => val.to_str().unwrap_or("").eq_ignore_ascii_case("bytes"),
        _ => false,
    };

    Ok(FileInfo { content_length, accept_ranges })
}