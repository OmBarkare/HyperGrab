use project1::dmserver;
use project1::odm;
use project1::odm::dowload_from_url_to;
use std::net::TcpListener;
use std::str::FromStr;
use std::{env, fs::File};
use url::Url;

fn main() {
    //Get environment variables
    let args: Vec<String> = env::args().collect();
    // let input_url = &args[1];
    let filename = &args[1];
    // let input_url = Url::parse(input_url).unwrap();
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
    let url = dmserver::handle_request(listener).unwrap();
    println!("Got download link: {:?}", url);
    let url = Url::from_str(&url).unwrap();


    match odm::get_file_info(&url) {
        Ok(file_info) => {
            dbg!(&file_info);

            let mut file = File::create(format!("{}/{}", dirs::download_dir().unwrap().to_str().unwrap(), filename)).unwrap();
            dowload_from_url_to(&url, &mut file);

            println!("Download complete!");
        },

        Err(e) => {
            println!("Error occured: {}", e);
        }
    }
}
