//Next Steps: Make request handlesr server on another thread, so that it is active and does not close after a single request is sent.

use project1::odm::download_chunk;
use project1::{dmserver, dmserver::RequestInfo, odm, odm::dowload_from_url_to};
use std::net::TcpListener;
use std::thread;
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

    match odm::get_file_info(&req_info) {
        Ok(file_info) => {
            dbg!(&file_info);

            let file_path = format!(
                "{}/{}",
                dirs::download_dir().unwrap().to_str().unwrap(),
                filename
            );
            
            //preallocating the space for file.
            File::create(&file_path).unwrap().set_len(file_info.size).unwrap();

            // let mut file = File::create(format!("{}/{}", dirs::download_dir().unwrap().to_str().unwrap(), filename)).unwrap();
            // dowload_from_url_to(&req_info, &mut file);
            let chunks = 4;
            let mut thread_handles = vec![];
            for i in 0..chunks {
                let mut start = file_info.size / chunks * i;
                let mut end = file_info.size / chunks * (i + 1) - 1;
                if i == chunks - 1 {
                    start = file_info.size / chunks * i;
                    end = file_info.size -1;
                }
                let req_info_clone = req_info.clone();
                let file_path_clone = file_path.clone();

                let handle = thread::spawn(move || {
                    println!("Opening thread {i}");
                    download_chunk(req_info_clone, start, end, &file_path_clone);
                });

                thread_handles.push(handle);
            }
            for handle in thread_handles {
                handle.join().unwrap();
            }
            println!("Download complete!");
        }



        Err(e) => {
            println!("Error occured: {}", e);
        }
    }
}
