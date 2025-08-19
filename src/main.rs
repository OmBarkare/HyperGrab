use std::error::Error;
use std::fs::File;
use std::io::Write;
use std::env;
use std::task::Context;
use url::Url;
use reqwest;


fn main() -> Result<(), Box<dyn Error>>{
    let args: Vec<String> = env::args().collect();
    let input_url = &args[1];
    let parsed_url = Url::parse(input_url).unwrap();
    let filename;
    
    if let Some(file_name) = parsed_url.path_segments().and_then(|s| s.last()) {
        println!("{}", file_name);
        filename = file_name;
    } else {
        panic!("Could Not Get file_name");
    }

    let resp = reqwest::blocking::get(parsed_url.as_str())?;
    let body = resp.bytes()?;

    let mut file = File::create(filename)?;
    file.write(&body[..])?;
    
    Ok(())
}
