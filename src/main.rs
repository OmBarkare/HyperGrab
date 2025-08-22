use project1::dmserver;
use project1::odm;
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

    //create file
    let mut pathname = odm::get_path(&url);

    //download to file
    if let Some(p) = dirs::download_dir() {
        pathname = format!("{}/{}", p.to_str().unwrap(), filename);
    } else {
        pathname = format!(
            "{}/{}",
            env::current_dir().unwrap().to_str().unwrap(),
            filename
        );
    }
    let mut file = File::create(&pathname).unwrap();
    odm::dowload_from_url_to(&url, &mut file);

    println!("saving to {}", &pathname);
}
