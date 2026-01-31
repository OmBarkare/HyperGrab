# HyperGrab

HyperGrab is an **experimental HTTP downloader** written in Rust to explore
concurrency models and file I/O.

---

## Project Status: Experimental

This is a **learning project** and not a production-grade download manager.
The focus is experimentation and understanding real-world complexity, not
feature completeness or reliability.

---

## Motivation

This project was built to gain hands-on experience with:
- Rust systems programming
- file I/O and buffering
- concurrency and async execution
- HTTP downloading and its edge cases

What initially seemed like a simple problem quickly revealed significant
complexity, especially around error handling and coordinating multiple
concurrent download tasks. Understanding these challenges was a primary goal
of the project.

---

## What It Currently Does

- Downloads files over HTTP
- Supports segmented downloads
- Includes an async-based implementation
- Integrates with a Chrome extension to intercept downloads

---

## Future Work

Possible future directions for this project include:

- Exploring a worker-pool-based design instead of spawning a fixed set of async tasks
- Improving error handling and retry logic
- Detecting unsupported HTTP range requests
- Adding basic progress reporting
- Exploring resumable downloads
- Simplifying the overall project structure

---

## Installation

Pre-built binaries are not available at this time.
You must build the project from source.

## How to Use

This project works by combining a **local Chrome extension** with the
**downloader service**. The Chrome extension intercepts browser
downloads and forwards the download metadata to a local server running on
`localhost`.

### Usage Flow

1. Load the Chrome extension
2. Start the Rust downloader service
3. Trigger a download in Chrome

---

### Step 1: Load the Chrome Extension

1. Open Google Chrome
2. Navigate to `chrome://extensions`
3. Enable **Developer mode**
4. Click **Load unpacked**
5. Select the `extension/chrome` directory from this repository

This extension intercepts browser download events and sends the download
information to a local server.

---

### Step 2: Start the Downloader Service

From the top-level project directory, run:

```bash
cargo run -p downloader-async
```

### Requirements
- Rust toolchain
- OpenSSL

Install OpenSSL on Debian-based systems:
```bash
sudo apt install libssl-dev
```
