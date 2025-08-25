pub mod odm {
    use reqwest::header::{ACCEPT, HeaderValue, USER_AGENT};
    use std::env;
    use std::{fs::File, io::Write};
    use url::Url;

    fn make_request(url: Url) -> reqwest::blocking::Response {
        let mut http_request_builder = reqwest::blocking::ClientBuilder::new();
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(USER_AGENT, HeaderValue::from_static("Mozilla/5.0"));
        headers.insert(ACCEPT, HeaderValue::from_static("*/*"));

        // let mut headers = HeaderMap::new();
        // headers.insert(USER_AGENT, HeaderValue::from_static("Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/138.0.0.0 Safari/537.36"));
        // headers.insert(ACCEPT, HeaderValue::from_static("text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8"));
        // headers.insert(ACCEPT_LANGUAGE, HeaderValue::from_static("en-US,en;q=0.9"));
        // headers.insert(ACCEPT_ENCODING, HeaderValue::from_static("gzip, deflate"));
        // headers.insert(CONNECTION, HeaderValue::from_static("keep-alive"));
        http_request_builder = http_request_builder.default_headers(headers);
        let client = http_request_builder.build().unwrap();
        let resp = client.get(url.as_str()).send().unwrap();

        resp
    }

    fn get_filename(url: &Url) -> Result<String, ()> {
        if let Some(filename) = url.path_segments().and_then(|s| s.last()) {
            Ok(filename.to_string())
        } else {
            panic!("No filename fonud in url");
        }
    }

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
        let body_bytes = make_request(url.clone()).bytes().unwrap();

        match file.write_all(&body_bytes[..]) {
            Ok(_) => (),
            Err(_) => panic!("Could Not write to file"),
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
    use std::{
        io::{Read, Write},
        net::TcpListener,
    };

    fn parse_for_dwlink(request: String) -> String {
        //seperate header and body
        let split_req: Vec<&str> = request.split("\r\n\r\n").collect();
        dbg!(&split_req);
        let body = split_req[1];
        let body = body.trim_matches('\0').trim();
        let link = format!("{body}");

        link
    }

    pub fn handle_request(listener: TcpListener) -> Result<String, ()> {
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
                return Ok(parse_for_dwlink(request));
            }
        }
        Err(())
    }
}
