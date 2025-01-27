use reqwest;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::{fs, io::Read, thread, time};

#[derive(Deserialize)]
struct Ip {
    ip: String,
}

#[derive(Serialize, Deserialize)]
struct Config {
    last_ip: String,
    webhook: String,
    interval: u64,
}

#[tokio::main]
async fn main() {
    let mut file = fs::OpenOptions::new()
        .read(true)
        .append(false)
        .open("/etc/ip_change_notifier/config.json")
        .expect("Configuration file not found in /etc/ip_change_notifier/config.json");

    let mut content = String::new();
    let _ = file.read_to_string(&mut content);

    let mut config: Config =
        serde_json::from_str(&content).expect("There's an error in the configuration file");

    let client = reqwest::Client::new();

    let mut payload = json!({"content":format!("> :computer: PC HAS TURNED ON\n> CURRENT IP: **{}**", &config.last_ip)});

    let _ = client
        .post(&config.webhook)
        .json(&payload)
        .send()
        .await
        .expect("Initial webhook didn't work");

    println!("INITIAL WEBHOOK SENT");

    loop {
        let request = reqwest::get("https://api.ipify.org?format=json")
            .await
            .expect("Fetch didn't work")
            .json::<Ip>()
            .await;

        if let Ok(response) = request {
            println!("CURRENT IP: {}", &config.last_ip);

            if config.last_ip != response.ip {
                println!("NEW IP: {}", &response.ip);

                payload = json!({"content":format!("> :warning: PUBLIC IP HAS CHANGED\n> OLD: **{0}**\n> NEW: **{1}**", &config.last_ip, &response.ip)});

                config.last_ip = response.ip;
                let parsed = serde_json::to_string(&config).expect("Couldn't parse to JSON");

                let _ = fs::write("/etc/ip_change_notifier/config.json", &parsed)
                    .expect("Couldn't write to configuration file");
                config = serde_json::from_str(&parsed)
                    .expect("There's an error in the new configuration file");

                if config.webhook != "" {
                    let _ = client
                        .post(&config.webhook)
                        .json(&payload)
                        .send()
                        .await
                        .expect("Webhook didn't work");

                    println!("WEBHOOK SENT");
                }
            }
        }

        thread::sleep(time::Duration::from_millis(config.interval));
    }
}
