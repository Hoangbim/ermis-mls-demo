use reqwest::blocking::Client;
use std::thread::sleep;
use std::time::Duration;

pub fn call_long_polling() {
    let client = Client::new();
    let url = "http://localhost:8080/long_poll";
    println!("Calling long-polling at: {}", url);
    loop {
        match client.get(url).send() {
            Ok(response) => {
                if let Ok(body) = response.text() {
                    println!("Received response: {}", body);
                }
            }
            Err(e) => {
                eprintln!("Error during long-polling: {}", e);
            }
        }
        // Đợi một khoảng thời gian trước khi gọi lại
        sleep(Duration::from_secs(1));
    }
}
