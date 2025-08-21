pub mod odm {
    use reqwest::header::{ACCEPT, HeaderValue, USER_AGENT};
    use std::env;
    use std::{fs::File, io::Write};
    use url::Url;

    pub fn make_request(url: Url) -> reqwest::blocking::Response {
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
        let http_request = http_request_builder.build().unwrap();
        let resp = http_request.get(url.as_str()).send().unwrap();

        resp
    }

    fn get_filename(url: &Url) -> Result<String, ()>{
        if let Some(filename) = url.path_segments().and_then(|s| s.last()) {
            Ok(filename.to_string())
        } else {
            panic!("No filename fonud in url");
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
