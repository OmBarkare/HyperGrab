//Next Steps: Make request handlesr server on another thread, so that it is active and does not close after a single request is sent.


use project1::{dmserver, odm, dmserver::RequestInfo, odm::dowload_from_url_to};
use std::net::TcpListener;
use std::{env, fs::File};
use std::thread;

fn main() {
    //Get environment variables
    let args: Vec<String> = env::args().collect();
    // let input_url = &args[1];
    let filename = &args[1];

    // let input_url = Url::parse(input_url).unwrap();
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
    
    let req_info: dmserver::RequestInfo;
    match dmserver::start_listening(listener) {
        Ok(r ) => {req_info = r},
        Err(e) => {
            req_info = RequestInfo {
                url: "#".to_string(),
                headers: std::collections::HashMap::from([("$".to_string(), "$".to_string())])
            };
            println!("{:?}",e)
        },
    }
    println!("Got download link: {:?}", &req_info);
    // let url = Url::from_str(&url).unwrap();


    match odm::get_file_info(&req_info) {
        Ok(file_info) => {
            dbg!(&file_info);

            let mut file = File::create(format!("{}/{}", dirs::download_dir().unwrap().to_str().unwrap(), filename)).unwrap();
            dowload_from_url_to(&req_info, file_info, &mut file);

            println!("Download complete!");
        },

        Err(e) => {
            println!("Error occured: {}", e);
        }
    }
}
