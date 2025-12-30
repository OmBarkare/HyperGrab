# HyperGrab
This is a download manager tool, which is still in developement.

## Installation
As of now, binaries are not available to download directly. You will have
to clone the repository and build from source.

after cloning this project, run this in the top level project directory
```bash
cargo run -p downloader-async
``` 
The project will start to build.
This project requires openSSL, install it using
```bash
sudo apt install libssl-dev
```
Then, your project should build and run.

## Usage
1. Go to chrome (this works only with chrome as of now)
2. go to manage extensions and turn on developer options
3. click 'load unpacked' and add the chrome extension under the extension directory this project

now, whenever you click download on chrome, it will cancel that download and download with the
download manager
