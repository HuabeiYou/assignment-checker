#![allow(dead_code, unused_variables)]
use mac_address;
use reqwest;
use std::{collections::HashMap, error, io, path::PathBuf};
mod constants;

pub fn run(config: Config) -> Result<(), Box<dyn error::Error>> {
    validate_files(&config.paths)?;
    let auth_resp = authenticate(&config)?;
    match auth_resp.get("message") {
        Some(v) => println!("{}", v),
        _ => {}
    };
    Ok(())
}

pub struct Config {
    pub lesson: String,
    pub phone: String,
    pub paths: Vec<PathBuf>,
}

impl Config {
    pub fn build(lesson: &str, phone: &String, files: Vec<&String>) -> Result<Self, io::Error> {
        let phone_clone = phone.clone();
        let curr_dir = std::env::current_dir()?;
        let paths_clone = files
            .into_iter()
            .map(|x| -> PathBuf {
                let dir: PathBuf = std::env::current_dir().unwrap();
                dir.join(x)
            })
            .collect::<Vec<PathBuf>>();
        Ok(Self {
            lesson: String::from(lesson),
            phone: phone_clone,
            paths: paths_clone,
        })
    }
}

#[tokio::main]
async fn authenticate(config: &Config) -> Result<HashMap<String, String>, reqwest::Error> {
    let mac = match mac_address::get_mac_address() {
        Ok(option) => match option {
            Some(value) => value.to_string(),
            None => String::from("unknown"),
        },
        Err(_) => String::from("unknown"),
    };
    let query_string = format!(
        "lesson={}&phone={}&mac={}",
        &config.lesson,
        &config.phone,
        urlencoding::encode(&mac)
    );
    let url = format!("{}?{}", constants::AUTH_ENDPOINT, query_string);
    let client = reqwest::Client::new();
    let resp = client.get(url.to_string()).send().await?;
    let data = resp.json::<HashMap<String, String>>().await?;
    Ok(data)
}

fn validate_files(paths: &Vec<PathBuf>) -> Result<(), io::Error> {
    for file in paths {
        let is_file = file.is_file();
        if !is_file {
            return Err(io::Error::new(io::ErrorKind::NotFound, "File not found"));
        }
        let file_size = file.metadata()?.len();
        if file_size > 1024 * 1000 {
            return Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "File is too large",
            ));
        }
    }
    Ok(())
}
