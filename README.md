# HyperGrab

HyperGrab is an **experimental, high-performance HTTP download manager** written in Rust to explore concurrency models, network protocols, and low-level file I/O optimization.

This is a learning project where I am learning about network protocols and server behaviour, how to handle different errors and throttling by the server to get maximum performance, manage concurrent tasks along with with kernel file I/O

---

## Brief Overview of Implementation

### 1. Shared-Work Queue
- Implement a **Shared work** (`VecDeque<Chunk>`). Faster workers automatically grab new chunks as they finish, preventing slow connections or stragglers from delaying the overall download.
- Configurable worker count via flags (`-w`, `--workers`).

### 2. Disk I/O
- Previously `tokio::fs::File::set_len()` was used which only changes the logical size of the file creating a sparse file, causing disk allocations to occur at the time of writing chunks to disk. Replace that with `fs4::FileExt::allocate()` which uses `posix_fallocate` under the hood to reserve space on disk
- instead of writing to disk on every return from `bytes_stream().next().await`, use `BufWriter` to buffer bytes in memory before making the write syscall. This reduces the number of context switches.

### 3. Protocol & Network Tuning
- Use HTTP/1.1 always, instead of trying for HTTP/2, so that each worker has their own contention window, do not suffer from HoL blocking due to other workers and are truly independent as their stream is a separate connection.
- Probes metadata and byte-range support using `Range: bytes=0-0` GET requests, as servers sometimes drop HEAD requests.
- Strips conflicting browser pseudo-headers and connection headers (`:path`, `Host`, `Range`, `Connection`) to prevent TLS and HTTP protocol rejections.

### 4. Rate-Limiting & Error Recovery
- Automatically pauses and retries failed chunks up to 5 times with exponential backoff on network errors.
- Detects HTTP `429 Too Many Requests` or `503 Service Unavailable` and respects server-mandated cooldown timers before resuming requests.

### 5. Filename Resolution
- Parsing of `Content-Disposition` headers. If that is not found, a default name (download.<file_type>) is given. File type is guessed using the `mime_guess` crate

---

## High-Level Architecture Overview

<p aligne="center">
    <img src="assets/HyperGrab-high-level-flow.svg" />
</p>

---

## Benchmark Results

### Setting Up For The Benchmark
Benchmarks were performed on a local nginx server which limits rate to 1m.
Here is the server config:
```
server {
	listen 443 ssl;
	http2 on;
	server_name localhost;
	
	ssl_certificate /etc/nginx/localhost.crt;
	ssl_certificate_key /etc/nginx/localhost.key;
	
	location /testfile.bin {
		root /var/www/speedtest;
		limit_rate 1m;
	}
}
```

- I created a certificate using
```
openssl req -x509 -nodes -days 365 -newkey rsa:2048 \
-keyout /etc/nginx/localhost.key \
-out /etc/nginx/localhost.crt \
-subj "/CN=localhost"
```

- To simulate latency in network, I used `tc`
`tc qdisc add dev lo root netem delay 40ms`

You will probably have to run above two commands with `sudo`

### Benchmark Results

```
================================================================================
BENCHMARK & CAPTURE SUMMARY
Timestamp: 2026-09-22 02:50:00
Target:    https://localhost/testfile.bin
================================================================================
Protocol   | Workers  | Avg Time   | Min / Max Time     | Avg Speed
--------------------------------------------------------------------------------
HTTP/1.1   | 4        | 32.36    s | 32.31s / 32.43s    | 3.09       MB/s
HTTP/1.1   | 6        | 20.16    s | 20.13s / 20.20s    | 4.96       MB/s
HTTP/1.1   | 8        | 17.32    s | 16.31s / 21.21s    | 5.77       MB/s
HTTP/2     | 4        | 78.27    s | 39.84s / 122.56s   | 1.28       MB/s
HTTP/2     | 6        | 81.36    s | 52.39s / 126.74s   | 1.23       MB/s
HTTP/2     | 8        | 82.90    s | 20.80s / 124.04s   | 1.21       MB/s
================================================================================

Captures Directory: /home/omomo/nomomon/projects/HyperGrab/scratch/
TLS Keylog File:    /tmp/sslkeylogfile.txt
```

#### Insights
- HTTP/1.1 download speed increases with increase in number of workers, which is intuitive as each worker opens a new connection
- HTTP/2 download speed has a large std deviation. This is because all workers share the same TCP connection, and hence share the same receive-window. This window is dynamically managed by the kernel's network stack. On good runs, the workers happen to pull bytes fast enough (somehow, still looking into how it happens. I think it has got to do with how the tasks get scheduled by the tokio scheduler, currently looking into it :) ) so the kernel keeps growing the sk_rcvbuf (the receive buffer size, which leads to increased receive-window size). On bad runs, the workers are not able to cross some maximum threshold of reading speed, and hence the sk_rcvbuf does not grow.

## What to implement next

- Download progress in cli
- Heuristic-based chunk sizing & worker scaling
- Pausable and resumable downloads across sessions

> [!NOTE]
> the code for attaching a keylogger and all of its implementation in tls.rs is written by AI (gemini 3.8 flash)
---

## Installation & Setup

### Prerequisites
- **Rust toolchain** (latest stable)

### Building from Source

```bash
git clone https://github.com/OmBarkare/HyperGrab.git
cd HyperGrab
cargo build --release
```

---

## How to Use

HyperGrab integrates with your browser via an extension that intercepts downloads and forwards the download URL and headers to the local Rust service.

### Step 1: Load the Browser Extension

#### For Google Chrome:
1. Open Chrome and navigate to `chrome://extensions`.
2. Enable **Developer mode** (top-right toggle).
3. Click **Load unpacked** and select the `extension/chrome` directory.

#### For Mozilla Firefox:
1. Open Firefox and navigate to `about:debugging` and go to **this firefox** tab.
2. Click **Load Temporary Add-on**.
3. Select any file inside the `extension/firefox` directory (e.g., `manifest.json`).

---

### Step 2: Start the Downloader Service

You will find the target directory in root of directory

```bash
# Default (4 workers)
`target/release/downloader-async`
# Custom worker count
`target/release/downloader-async -w <worker_cout>`

# OR use this for usage Info
`target/release/downloader-async --help`
```

### Step 3: Download
Click any download link in your browser. The extension will intercept the download and pass it to HyperGrab!
