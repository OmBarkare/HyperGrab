//Next Steps: Make request handlesr server on another thread, so that it is active and does not close after a single request is sent.

use downloader_mt::odm::{downloadv1};
use downloader_mt::{dmserver, dmserver::RequestInfo, odm};
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();     
    let filename = &args[1];
    let listener_blocking = std::net::TcpListener::bind("127.0.0.1:7878").unwrap();
    let req_info: dmserver::RequestInfo;

    match dmserver::start_listening(listener_blocking) {
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
    let resp_info = odm::get_resp_info(&req_info);

    downloadv1(&req_info, resp_info, filename);
}
