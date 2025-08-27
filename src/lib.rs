pub mod odm {
    use anyhow::Ok;
    use reqwest::header::{HeaderValue, ACCEPT, REFERER, USER_AGENT};
    use reqwest::blocking::Client;
    use std::env;
    use std::{fs::File, io::Write};
    use url::Url;

    fn make_http_client() -> Result<reqwest::blocking::Client, anyhow::Error> {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(USER_AGENT, HeaderValue::from_static("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/125.0.0.0 Safari/537.36"));
        headers.insert(ACCEPT, HeaderValue::from_static("*/*"));
        headers.insert(REFERER, HeaderValue::from_static("https://pixabay.com/"));

        let client = reqwest::blocking::Client::builder().default_headers(headers).build()?;

        Ok(client)
    }

    fn make_request(url: Url,client: Client) -> reqwest::blocking::Response {
        let resp = client.get(url.as_str()).send().unwrap();

        resp
    }

    fn get_filename(url: &Url) -> Result<String, anyhow::Error> {
        if let Some(filename) = url.path_segments().and_then(|s| s.last()) {
            return Ok(filename.to_string())
        } else {
            panic!("No filename fonud in url");
        }
    }

    #[derive(Debug)]
    pub struct FileInfo {
        pub size: u64,
        pub supports_ranges: bool,
    }

    pub fn get_file_info(url: &Url) -> Result<FileInfo, anyhow::Error> {
        let mut http_request_builder = reqwest::blocking::ClientBuilder::new();
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(USER_AGENT, HeaderValue::from_static("Mozilla/5.0"));
        headers.insert(ACCEPT, HeaderValue::from_static("*/*"));

        http_request_builder = http_request_builder.default_headers(headers);
        let client = http_request_builder.build()?;

        let resp = client.head(url.as_str()).send()?;

        if !resp.status().is_success() {
            anyhow::bail!("Server responded with error {}", resp.status());
        } else {
            let headers = resp.headers();
            let content_len:u64 = headers.get("CONTENT_LENGTH").and_then(|v| v.to_str().ok()).and_then(|v| v.trim().parse().ok()).unwrap_or(0);

            let supports_ranges = headers.get("ACCEPT_RANGES").unwrap().to_str().unwrap();
            let supports_ranges = if supports_ranges == "bytes" {true} else {false};

            if content_len == 0 {
                anyhow::bail!("content length is zero");
            }

            Ok(FileInfo { size: content_len, supports_ranges: supports_ranges })
        }
    }

    pub fn dowload_from_url_to(url: &Url, file: &mut File) {
        let client = make_http_client().unwrap();
        let body_bytes = make_request(url.clone(), client).bytes().unwrap();

        file.write_all(&body_bytes[..]).unwrap();
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
    use std::{
        io::{Read, Write},
        net::TcpListener,
    };
    use anyhow::{Error, Ok};
    use reqwest::header;
    use serde::{Deserialize, Serialize};
    use serde_json;

    #[derive(Deserialize, Serialize, Debug)]
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
            },
            None => {
                panic!("Invalid request");
            }
        }
        Ok(req_info)
    }

    pub fn handle_req(listener: TcpListener) -> Result<RequestInfo, anyhow::Error> {
        for stream in listener.incoming() {
            let mut stream = stream.unwrap();
            let mut buffer = vec![0u8; 2048];

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

                return  Ok(req_info);
            }
        }
        Err(anyhow::anyhow!("No valid req recieved, Inside handle_req"))
    }
}
