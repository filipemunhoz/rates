use std::{time::Instant, fs::File};
use reqwest::Client;
use serde::Deserialize;
use std::fs;
use std::fs::OpenOptions;
use std::io::prelude::*;
use futures::{stream, StreamExt};

const CONCURRENT_REQUESTS: usize = 2;

#[derive(Deserialize)]
struct Currency {
    result: f32,
    query: Query,
}

#[derive(Deserialize)]
struct Query {
    from: String,
}

#[tokio::main]  
async fn main() -> Result<(), reqwest::Error> {

    let start = Instant::now();
    let mut outputs:Vec<String> = Vec::new();
    let path = std::path::Path::new("/home/fcmunhoz/.rates"); 

    let urls = vec!["https://api.apilayer.com/exchangerates_data/convert?to=BRL&from=USD&amount=1",
                                "https://api.apilayer.com/exchangerates_data/convert?to=BRL&from=EUR&amount=1",
                                    "https://api.apilayer.com/exchangerates_data/convert?to=BRL&from=GBP&amount=1"];

    let client = Client::new();
    let bodies = stream::iter(urls)
    .map(|url| {
        let client = &client;
        async move {
            let resp = client.get(url)
            .header("apikey", "api")
            .send().await?;
            resp.bytes().await
        }
    })
    .buffered(CONCURRENT_REQUESTS);

    bodies
        .for_each(|b| {
            match b {
                Ok(b) => {
                    outputs.push(output(&b));
                },
                Err(_e) => eprintln!("Not connected"),
            }
            async {}
        })
        .await;

    if outputs.is_empty() {
        return Ok(());
    }

    if path.exists() {
        fs::remove_file(path).unwrap();
    }
      
    File::create(path).unwrap();
    let mut file = OpenOptions::new()
        .write(true)
        .append(true)
        .open(path)
        .unwrap();     

    for i in outputs {
        print!("{i}");
        file.write_all(i.as_bytes()).ok();
    }
    let duration = start.elapsed().as_millis();

    println!("Duration {}ms", duration);
    Ok(())
}

fn output(b: &[u8]) -> String {
    let currency: Currency = serde_json::from_slice(b).unwrap();
    format!("{} {:.2}   ", emoji_currency(currency.query.from), currency.result)
}

fn emoji_currency(currency: String) -> String {
    match currency.as_str() {
        "USD" => format!("{}",'💵'),
        "EUR" => format!("{}",'💶'),
        "GBP" => format!("{}",'💷'),
        _ => format!("{}", '❓'),        
    }   
}