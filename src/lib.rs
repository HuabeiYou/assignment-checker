mod arguments;
mod constants;
use arguments::LESSON;
use constants::AUTH_ENDPOINT;
use mac_address;
use reqwest::multipart;
use serde::Serialize;
use spinners::{Spinner, Spinners};
use std::{collections::HashMap, error, fs, io, path::PathBuf};

pub fn run(config: Config) -> Result<(), Box<dyn error::Error>> {
    validate_files(&config.paths)?;
    let auth_resp = authenticate(&config)?;
    if let Some(message) = auth_resp.get("message") {
        println!("{}", message)
    };
    if !auth_resp.contains_key("RunnerLocation") {
        return Ok(());
    };
    let mut sp = Spinner::new(Spinners::Dots12, "Validating test files...".into());
    let jobs = config
        .paths
        .into_iter()
        .map(|x| UploadJob::build(&auth_resp, x).unwrap())
        .collect::<Vec<UploadJob>>();
    let uploaded_files = upload(jobs).expect("Please check your internet connection.");
    sp.stop_with_message("Validation success.".into());
    let runner_location = auth_resp.get("RunnerLocation").unwrap();
    let mut sp = Spinner::new(
        Spinners::Dots12,
        "Running the test... This may take a while, thank you for your patience.".into(),
    );
    let test_result = run_test(runner_location, uploaded_files)?;
    println!("{}", test_result);
    sp.stop_with_message("Test finished".into());
    Ok(())
}

#[tokio::main]
async fn upload(jobs: Vec<UploadJob>) -> Result<Vec<HashMap<String, String>>, reqwest::Error> {
    let mut results = Vec::new();
    for job in jobs {
        let form = multipart::Form::new()
            .text("key", String::from(&job.key))
            .text("OSSAccessKeyId", job.oss_access_key)
            .text("policy", job.policy)
            .text("Signature", job.signature)
            .part(
                "file",
                multipart::Part::bytes(job.data).file_name(job.file_name),
            );
        let client = reqwest::Client::new();
        let resp = client.post(job.destination).multipart(form).send().await?;
        resp.error_for_status()?;
        let mut result = HashMap::new();
        result.insert("key".into(), job.key);
        result.insert("bucket".into(), job.bucket);
        results.push(result);
    }
    Ok(results)
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
    let url = format!("{}?{}", AUTH_ENDPOINT, query_string);
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

#[derive(Debug)]
struct UploadJob {
    key: String,
    bucket: String,
    destination: String,
    oss_access_key: String,
    policy: String,
    signature: String,
    file_name: String,
    data: Vec<u8>,
}
impl UploadJob {
    fn build(
        params: &HashMap<String, String>,
        path: PathBuf,
    ) -> Result<Self, Box<dyn error::Error>> {
        let file_name = String::from(path.file_name().unwrap().to_str().unwrap());
        let key = format!("{}/{}", params.get("Dir").unwrap(), file_name);
        let data = fs::read(path).unwrap();
        Ok(Self {
            key,
            bucket: params.get("Bucket").unwrap().into(),
            destination: String::from("https://savvyuni-materials.oss-accelerate.aliyuncs.com"),
            oss_access_key: params.get("OSSAccessKeyId").unwrap().to_string(),
            policy: params.get("Policy").unwrap().to_string(),
            signature: params.get("Signature").unwrap().to_string(),
            file_name,
            data,
        })
    }
}

pub struct Config {
    pub lesson: String,
    pub phone: String,
    pub paths: Vec<PathBuf>,
}

impl Config {
    pub fn build(lesson: &str, phone: &String, files: Vec<&String>) -> Result<Self, io::Error> {
        let phone_clone = phone.clone();
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

#[derive(Debug, Serialize)]
struct RequestBody {
    lesson: String,
    files: Vec<HashMap<String, String>>,
}

#[tokio::main]
async fn run_test(
    server: &String,
    files: Vec<HashMap<String, String>>,
) -> Result<String, reqwest::Error> {
    let body = RequestBody {
        lesson: String::from(LESSON),
        files,
    };
    let client = reqwest::Client::new();
    let resp = client.post(server).json(&body).send().await?;
    let data = resp.text().await?;
    Ok(data)
}
