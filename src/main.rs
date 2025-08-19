use std::error::Error;
use std::fs::File;
use std::io::Write;
use reqwest;


fn main() -> Result<(), Box<dyn Error>>{
    println!("Hello, world!");

    let resp = reqwest::blocking::get("https://www.rust-lang.org")?;
    let body = resp.text()?;
    let mut file = File::create("foo.txt")?;
    file.write(body.as_bytes())?;
    println!("{}", body);

    Ok(())
}
