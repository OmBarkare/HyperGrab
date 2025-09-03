//Next Steps: Make request handlesr server on another thread, so that it is active and does not close after a single request is sent.

use project1::odm::{download_chunk, downloadv1};
use project1::{dmserver, dmserver::RequestInfo, odm, odm::dowload_from_url_to};
use std::net::TcpListener;
use std::thread::{self, sleep};
use std::time::Duration;
use std::{env, fs::File};

fn main() {
    //Get environment variables
    let args: Vec<String> = env::args().collect();
    // let input_url = &args[1];
    let filename = &args[1];

    // let input_url = Url::parse(input_url).unwrap();
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();

    let req_info: dmserver::RequestInfo;
    match dmserver::start_listening(listener) {
        Ok(r) => req_info = r,
        Err(e) => {
            req_info = RequestInfo {
                url: "#".to_string(),
                headers: std::collections::HashMap::from([("$".to_string(), "$".to_string())]),
            };
            println!("{:?}", e)
        }
    }
    println!("Got download link: {:?}", &req_info);
    // let url = Url::from_str(&url).unwrap();

    let resp_info = odm::get_resp_info(&req_info);

    downloadv1(&req_info, resp_info, filename);

    // let version = file_info.unwrap().version;
    // match version {
    //     http::Version::HTTP_09 => {
    //         println!("Http Version 0.9");
    //     }

    //     http::Version::HTTP_10 => {
    //         println!("Http Version 1.0");
    //     }

    //     http::Version::HTTP_11 => {
    //         println!("Http Version 1.1");
    //     }

    //     http::Version::HTTP_2 => {
    //         println!("Http Version 2");
    //     }

    //     http::Version::HTTP_3 => {
    //         println!("Http Version 3");
    //     }

    //     _ => {
    //         panic!("Unknown http version");
    //     }
    // }

}
