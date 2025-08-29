pub mod odm {
    use anyhow;
    use reqwest::header::{HeaderName, HeaderValue};
    use std::env;
    use std::io::{Seek, SeekFrom};
    use std::str::FromStr;
    use std::{fs::File, io::Write};
    use url::Url;

    use crate::dmserver::RequestInfo;

    fn make_http_client(
        req_info: &RequestInfo,
    ) -> Result<reqwest::blocking::Client, anyhow::Error> {

        let mut http_request_builder = reqwest::blocking::ClientBuilder::new();
        let mut headers = reqwest::header::HeaderMap::new();

        let browser_headers = &req_info.headers;
        for (h, v) in browser_headers {
            if h.eq_ignore_ascii_case("host")
                || h.eq_ignore_ascii_case("content-length")
                || h.eq_ignore_ascii_case("transfer-encoding")
            {
                continue;
            }

            headers.insert(
                HeaderName::from_str(h).unwrap(),
                HeaderValue::from_str(v).unwrap(),
            );
        }

        // println!("THESE ARE NEW HEADERS: ");
        // dbg!(&headers);

        http_request_builder = http_request_builder.default_headers(headers);
        let client = http_request_builder.timeout(None).build()?;

        Ok(client)
    }

    fn make_http_client_async(req_info: &RequestInfo) -> Result<reqwest::Client, anyhow::Error> {
        let mut http_request_builder = reqwest::ClientBuilder::new();
        let mut headers = reqwest::header::HeaderMap::new();

        let browser_headers = &req_info.headers;
        for (h, v) in browser_headers {
            if h.eq_ignore_ascii_case("host")
                || h.eq_ignore_ascii_case("content-length")
                || h.eq_ignore_ascii_case("transfer-encoding")
            {
                continue;
            }

            headers.insert(
                HeaderName::from_str(h).unwrap(),
                HeaderValue::from_str(v).unwrap(),
            );
        }

        // println!("THESE ARE NEW HEADERS: ");
        // dbg!(&headers);

        http_request_builder = http_request_builder.default_headers(headers);
        let client = http_request_builder.build()?;

        Ok(client)
    }

    fn get_filename(url: &Url) -> Result<String, anyhow::Error> {
        if let Some(filename) = url.path_segments().and_then(|s| s.last()) {
            return Ok(filename.to_string());
        } else {
            panic!("No filename fonud in url");
        }
    }

    #[derive(Debug)]
    pub struct FileInfo {
        pub size: u64,
        pub supports_ranges: bool,
    }

    pub fn get_file_info(req_info: &RequestInfo) -> Result<FileInfo, anyhow::Error> {
        let client = make_http_client(&req_info).unwrap();
        let resp = client.head(req_info.url.clone()).send()?;

        if !resp.status().is_success() {
            anyhow::bail!("Server responded with error {}", resp.status());
        } else {
            let headers = resp.headers();
            let content_len: u64 = headers
                .get("CONTENT-LENGTH")
                .and_then(|v| v.to_str().ok())
                .and_then(|v| v.trim().parse().ok())
                .unwrap_or(0);

            println!("Content-length: {}", content_len);
            println!(
                "accept-ranges: {}",
                headers.get("ACCEPT-RANGES").unwrap().to_str().unwrap()
            );
            let supports_ranges = headers.get("ACCEPT-RANGES").unwrap().to_str().unwrap();
            let supports_ranges = if supports_ranges.eq_ignore_ascii_case("bytes") {
                true
            } else {
                false
            };

            if content_len == 0 {
                anyhow::bail!("content length is zero");
            }

            Ok(FileInfo {
                size: content_len,
                supports_ranges: supports_ranges,
            })
        }
    }

    pub fn dowload_from_url_to(req_info: &RequestInfo, file: &mut File) {
        let client = make_http_client(req_info).unwrap();
        let resp = client.get(req_info.url.clone()).send().unwrap();
        let body_bytes = resp.bytes().unwrap();

        file.write_all(&body_bytes[..]).unwrap();
    }

    pub fn download_chunk(
        mut req_info: RequestInfo,
        start: u64,
        end: u64,
        file_path: &String,
    ) -> Result<(), ()> {
        let mut headers = req_info.headers.clone();
        headers.insert("Range".to_string(), format!("bytes={start}-{end}"));
        req_info.headers = headers;
        let client = make_http_client(&req_info).unwrap();
        // println!("SENDING CHUNK DOWNLOAD WITH HEADERS: {:?}", &req_info.headers);
        let resp = client.get(&req_info.url).send().unwrap();

        match resp.headers().get("Content-Range") {
            Some(cr) => {
                println!("{:?}", &resp.headers().get("Content-Range"));
                let resp_bytes = resp.bytes().unwrap();

                //Open the file with write permissions
                let mut file = std::fs::OpenOptions::new()
                    .write(true)
                    .open(file_path)
                    .unwrap();
                //seek the handler to the chunk size
                file.seek(SeekFrom::Start(start)).unwrap();

                file.write_all(&resp_bytes).unwrap();

                return Ok(());
            }
            None => {
                return Err(());
            }
        }
    }

    pub fn get_path(url: &Url) -> String {
        let filename = get_filename(url).unwrap();
        let pathname;
        if let Some(p) = dirs::download_dir() {
            pathname = format!("{}/{}", p.to_str().unwrap(), filename);
        } else {
            pathname = format!(
                "{}/{}",
                env::current_dir().unwrap().to_str().unwrap(),
                filename
            );
        }
        pathname
    }
}

pub mod dmserver {
    use anyhow::{Error, Ok};
    use serde::{Deserialize, Serialize};
    use serde_json;
    use std::{
        clone,
        io::{Read, Write},
        net::TcpListener,
        thread,
    };

    #[derive(Deserialize, Serialize, Debug, Clone)]
    pub struct RequestInfo {
        pub url: String,
        pub headers: std::collections::HashMap<String, String>,
    }

    fn parse_req(request: String) -> Result<RequestInfo, anyhow::Error> {
        println!("inside parse_req");
        let req_info: RequestInfo;

        //seperate header and body
        match request.split_once("\r\n\r\n") {
            Some((_, body)) => {
                let body = body.trim_matches('\0').trim();
                // println!("match of parse_req, before serde_json");
                req_info = serde_json::from_str(body)?;
                // println!("after serde_json");
            }
            None => {
                panic!("Invalid request");
            }
        }
        Ok(req_info)
    }
    //make a struct which has the handle and the reciever for this listener thread
    pub fn start_listening(listener: TcpListener) -> Result<RequestInfo, anyhow::Error> {
        for stream in listener.incoming() {
            let mut stream = stream.unwrap();
            let mut buffer = vec![0u8; 16384];
            // let mut buffer = BufReader::new();
            let _ = stream.read(&mut buffer).unwrap();

            let request = String::from_utf8(buffer).unwrap();

            if request.starts_with("OPTION") {
                let response = "HTTP/1.1 200 OK\r\n\
                        Access-Control-Allow-Origin: *\r\n\
                        Access-Control-Allow-Headers: Content-Type\r\n\
                        Access-Control-Allow-Methods: POST\r\n\r\n";

                let _ = stream.write(response.as_bytes()).unwrap();
                println!("Got OPTIONS request");
            } else if request.starts_with("POST") {
                println!("Got POST request");

                let req_info = parse_req(request)?;

                return Ok(req_info);
            }
        }
        Err(anyhow::anyhow!("No valid req recieved, Inside handle_req"))
    }
}
