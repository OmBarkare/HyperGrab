use std::{env, fs::File};
use project1::odm;
use url::Url;


fn main() {
    //Get environment variables
    let args: Vec<String> = env::args().collect();
    let input_url = &args[1];
    // let filename = &args[2];
    let input_url = Url::parse(input_url).unwrap();

    //create file
    let pathname = odm::get_path(&input_url);
    let mut file = File::create(&pathname).unwrap();

    //download to file
    odm::dowload_from_url_to(&input_url, &mut file);

    println!("saving to {}", &pathname);
}
