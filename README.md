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

### Requirements
- Rust toolchain
- OpenSSL

Install OpenSSL on Debian-based systems:
```bash
sudo apt install libssl-dev
