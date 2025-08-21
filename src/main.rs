use std::error::Error;
use std::fs::File;
use std::io::{copy, Write};
use std::env;
use reqwest::header::{HeaderValue, ACCEPT, USER_AGENT};
use url::Url;
use reqwest::{self};
use dirs;
use std::path::Path;


fn main() -> Result<(), Box<dyn Error>>{
    let args: Vec<String> = env::args().collect();
    let input_url = &args[1];
    let filename = &args[2];
    // let filename;
    let parsed_url = Url::parse(input_url).unwrap();
    


    //Get Filename From url
    // if let Some(file_name) = parsed_url.path_segments().and_then(|s| s.last()) {
    //     println!("{}", file_name);
    //     filename = file_name;
    // } else {
    //     panic!("Could Not Get file_name");
    // }

    //Make Get Request To Server
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
    let http_request = http_request_builder.build()?;
    let resp = http_request.get(parsed_url.as_str()).send()?;

    // let resp = reqwest::blocking::get(parsed_url.as_str())?;
    println!("{}", resp.status());

    //parse the response and store it in a file with appropriate extension, in downloads folder
    let body = resp.bytes()?;

    //Deciding path
    let pathname;
    if let Some(p) = dirs::download_dir() {
        pathname = format!("{}/{}", p.to_str().unwrap(), filename);
    } else {
        pathname = format!("{}/{}", env::current_dir().unwrap().to_str().unwrap(), filename);
    }

    
    //Writing bytes to file
    let p= Path::new(&pathname);
    let mut file = File::create(p)?;
    file.write_all(&body[..])?;

    println!("saving to {}", pathname);
    Ok(())
}
