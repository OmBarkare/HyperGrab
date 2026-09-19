use std::{
    collections::{HashMap, VecDeque},
    str::FromStr,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};
use fs2::FileExt;
use futures::{StreamExt, future::join_all};
use reqwest::{
    self, Client, ClientBuilder,
    header::{HeaderMap, HeaderName, HeaderValue},
};
use tokio::{
    fs::{OpenOptions},
    io::{AsyncSeekExt, AsyncWriteExt, BufWriter},
};

const BASE_BACKOFF: Duration = Duration::from_secs(1);
const MAX_BACKOFF: Duration = Duration::from_secs(16);
const MAX_RETRIES: u32 = 5;
const WRITE_BUFFER_SIZE: usize = 256 * 1024; // 256 KB write buffer

/// A struct to store info we get from a head request
/// currently, it is assumed that the server accepts ranges so there is no
/// fallback if the server doesnt, so the accept_ranges field isnt used yet
pub struct FileInfo {
    pub content_length: u64,
    pub accept_ranges: bool,
    pub file_name: String,
}

#[derive(Debug, Clone)]
pub struct Chunk {
    pub id: usize,
    pub start: u64,
    pub end: u64,
    pub retries: u32,
}

/// Function that sends a 1-byte range request (Range: bytes=0-0) to probe
/// file metadata and range support without triggering HEAD request blocks.
pub async fn get_file_info(client: Client, url: &str) -> Result<FileInfo, anyhow::Error> {
    let resp = client.get(url).header("Range", "bytes=0-0").send().await?;

    let status = resp.status();
    let headers = resp.headers();

    let (content_length, accept_ranges) = if status == reqwest::StatusCode::PARTIAL_CONTENT {
        let total_size = headers
            .get("content-range")
            .and_then(|v| v.to_str().ok())
            .and_then(|cr| cr.split('/').nth(1))
            .and_then(|total| total.parse::<u64>().ok())
            .unwrap_or(0);

        (total_size, true)
    } else {
        let length = headers
            .get("content-length")
            .and_then(|v| v.to_str().ok())
            .and_then(|cl| cl.parse::<u64>().ok())
            .unwrap_or(0);

        (length, false)
    };

    println!("CONTENT_LENGTH (inside get_file_info): {}", content_length);
    println!("ACCEPT_RANGES: {}", accept_ranges);

    let file_name = resolve_filename(headers);

    Ok(FileInfo {
        content_length,
        accept_ranges,
        file_name,
    })
}

/// makes and returns a client with headers that you pass as a
/// hasmap
pub fn make_default_client(header_hashmap: &HashMap<String, String>) -> Client {
    let mut head_map = HeaderMap::new();
    let client_builder = ClientBuilder::new();
    for (key, val) in header_hashmap {
        if key.starts_with(':')
            || key.eq_ignore_ascii_case("host")
            || key.eq_ignore_ascii_case("content-length")
            || key.eq_ignore_ascii_case("transfer-encoding")
            || key.eq_ignore_ascii_case("connection")
            || key.eq_ignore_ascii_case("upgrade")
            || key.eq_ignore_ascii_case("range")
        {
            continue;
        }

        head_map.insert(
            HeaderName::from_str(key).unwrap(),
            HeaderValue::from_str(val).unwrap(),
        );
    }

    client_builder.http1_only().default_headers(head_map).build().unwrap()
}

pub fn create_chunks(content_length: u64, chunk_size: u64) -> VecDeque<Chunk> {
    let mut queue = VecDeque::new();
    let mut start = 0;
    let mut id = 0;

    while start < content_length {
        let end = (start + chunk_size - 1).min(content_length - 1);
        queue.push_back(Chunk {
            id,
            start,
            end,
            retries: 0,
        });
        start = end + 1;
        id += 1;
    }

    queue
}

fn calculate_backoff(retries: u32, headers: Option<&HeaderMap>) -> Duration {
    if let Some(h) = headers
        && let Some(retry_after) = h.get("retry-after").and_then(|v| v.to_str().ok())
        && let Ok(seconds) = retry_after.trim().parse::<u64>()
    {
        return Duration::from_secs(seconds).min(MAX_BACKOFF);
    }

    let multiplier = 2u64.saturating_pow(retries);
    let delay_millis = BASE_BACKOFF.as_millis().saturating_mul(multiplier as u128);
    Duration::from_millis(delay_millis as u64).min(MAX_BACKOFF)
}

fn handle_chunk_failure(
    mut chunk: Chunk,
    reason: &str,
    worker_id: usize,
    headers: Option<&HeaderMap>,
    queue: &Arc<Mutex<VecDeque<Chunk>>>,
    next_request_time: &Arc<Mutex<Instant>>,
) {
    if chunk.retries < MAX_RETRIES {
        let backoff = calculate_backoff(chunk.retries, headers);
        eprintln!(
            "Worker {worker_id} {reason} on chunk {}. Backing off for {:?} (retry {}/{})",
            chunk.id,
            backoff,
            chunk.retries + 1,
            MAX_RETRIES
        );

        {
            let mut next = next_request_time.lock().unwrap();
            let resume_at = Instant::now() + backoff;
            *next = (*next).max(resume_at);
        }

        chunk.retries += 1;
        queue.lock().unwrap().push_back(chunk);
    } else {
        eprintln!(
            "Worker {worker_id} chunk {} failed permanently after {MAX_RETRIES} retries ({reason})",
            chunk.id
        );
    }
}

pub async fn download_worker(
    worker_id: usize,
    client: Client,
    url: String,
    file_path: String,
    queue: Arc<Mutex<VecDeque<Chunk>>>,
    next_request_time: Arc<Mutex<Instant>>,
) {
    loop {
        let chunk = {
            let mut lock = queue.lock().unwrap();
            lock.pop_front()
        };

        let chunk = match chunk {
            Some(c) => c,
            None => {
                println!("Worker {worker_id} finished: queue empty.");
                break;
            }
        };

        println!(
            "Worker {worker_id} started chunk {}: bytes {}-{}",
            chunk.id, chunk.start, chunk.end
        );

        let delay = {
            let next = next_request_time.lock().unwrap();
            let now = Instant::now();
            let scheduled = *next;
            scheduled.saturating_duration_since(now)
        };

        if delay > Duration::ZERO {
            tokio::time::sleep(delay).await;
        }

        let range = format!("bytes={}-{}", chunk.start, chunk.end);
        let resp = match client.get(&url).header("Range", range).send().await {
            Ok(r) => r,
            Err(e) => {
                handle_chunk_failure(
                    chunk,
                    &format!("network error: {e}"),
                    worker_id,
                    None,
                    &queue,
                    &next_request_time,
                );
                continue;
            }
        };

        let status = resp.status();
        if status != reqwest::StatusCode::PARTIAL_CONTENT {
            handle_chunk_failure(
                chunk,
                &format!("received HTTP {status}"),
                worker_id,
                Some(resp.headers()),
                &queue,
                &next_request_time,
            );
            continue;
        }

        let mut file = match OpenOptions::new().write(true).open(&file_path).await {
            Ok(f) => f,
            Err(e) => {
                handle_chunk_failure(
                    chunk,
                    &format!("file open error: {e}"),
                    worker_id,
                    None,
                    &queue,
                    &next_request_time,
                );
                continue;
            }
        };

        if let Err(e) = file.seek(std::io::SeekFrom::Start(chunk.start)).await {
            handle_chunk_failure(
                chunk,
                &format!("file seek error: {e}"),
                worker_id,
                None,
                &queue,
                &next_request_time,
            );
            continue;
        }

        let mut writer = BufWriter::with_capacity(WRITE_BUFFER_SIZE, file);
        let mut bytes_stream = resp.bytes_stream();
        let mut downloaded_bytes: u64 = 0;
        let mut write_err = false;

        while let Some(item) = bytes_stream.next().await {
            match item {
                Ok(bytes) => {
                    downloaded_bytes += bytes.len() as u64;
                    if let Err(e) = writer.write_all(&bytes).await {
                        eprintln!("Worker {worker_id} file write error: {e}");
                        write_err = true;
                        break;
                    }
                }
                Err(e) => {
                    eprintln!("Worker {worker_id} stream error: {e}");
                    write_err = true;
                    break;
                }
            }
        }

        if !write_err && let Err(e) = writer.flush().await {
            eprintln!("Worker {worker_id} file flush error: {e}");
            write_err = true;
        }

        if write_err {
            handle_chunk_failure(
                chunk,
                "stream write error",
                worker_id,
                None,
                &queue,
                &next_request_time,
            );
        } else {
            println!(
                "Worker {worker_id} completed chunk {} ({downloaded_bytes} bytes)",
                chunk.id
            );
        }
    }
}

pub async fn spawn_download_tasks(
    client: Client,
    url: &str,
    file_info: FileInfo,
    file_path: &str,
    num_workers: usize,
) {
    let start_time = Instant::now();

    let file = std::fs::File::create(file_path).unwrap();
    if let Err(e) = file.allocate(file_info.content_length) {
        eprint!("Could not allocate disk space: {e}");
        panic!();
    }
    drop(file);

    const CHUNK_SIZE: u64 = 16 * 1024 * 1024; // 16MB
    let chunks = create_chunks(file_info.content_length, CHUNK_SIZE);
    let total_chunks = chunks.len();
    println!(
        "Total chunks to download: {} (using {} workers)",
        total_chunks, num_workers
    );

    let queue = Arc::new(Mutex::new(chunks));
    let next_request_time = Arc::new(Mutex::new(Instant::now()));
    let mut handles = Vec::new();

    for worker_id in 0..num_workers {
        let client = client.clone();
        let url = url.to_string();
        let file_path = file_path.to_string();
        let queue = queue.clone();
        let next_request_time = next_request_time.clone();

        let handle = tokio::spawn(download_worker(
            worker_id,
            client,
            url,
            file_path,
            queue,
            next_request_time,
        ));
        handles.push(handle);
    }

    println!("Joining all Handles...");
    join_all(handles).await;
    let elapsed = start_time.elapsed();
    println!("Download finished in {:.2} seconds", elapsed.as_secs_f64());
}

fn resolve_filename(headers: &HeaderMap) -> String {
    headers
        .get("content-disposition")
        .and_then(|v| v.to_str().ok())
        .and_then(|cd| cd.split("filename=").nth(1))
        .map(|name| name.trim_matches('"').trim_matches(';').trim().to_string())
        .filter(|name| !name.is_empty())
        .unwrap_or_else(|| fallback_filename(headers))
}

fn fallback_filename(headers: &HeaderMap) -> String {
    let ext = headers
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .and_then(|ct| ct.split(';').next())
        .map(str::trim)
        .and_then(|mime| mime_guess::get_mime_extensions_str(mime))
        .and_then(|exts| exts.first().copied());

    match ext {
        Some(extension) => format!("download.{extension}"),
        None => "download".to_string(),
    }
}
