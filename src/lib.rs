mod authentication;
mod constants;
mod files;

use crate::authentication::authenticate;
use crate::authentication::{AuthError, AuthResp};
use crate::files::{upload, validate_files, FileDescription, UploadJob};
use serde::Serialize;
use spinners::{Spinner, Spinners};
use std::{error, io, path::PathBuf};

pub fn run(config: Config) -> Result<AnalyticBody, Box<dyn error::Error>> {
    validate_files(&config.paths)?;
    let auth_resp = authenticate(&config)?;
    if auth_resp.failed() {
        return Err(Box::new(AuthError::new(auth_resp.message().into())));
    };
    let mut sp = Spinner::new(Spinners::Dots12, "Validating test files...".into());
    let jobs = config
        .paths
        .into_iter()
        .map(|x| UploadJob::build(&auth_resp, x).unwrap())
        .collect::<Vec<UploadJob>>();
    let uploaded_files = upload(jobs).expect("Please check your internet connection.");
    sp.stop_with_message("Validation success.".into());
    let mut sp = Spinner::new(
        Spinners::Dots12,
        "Running the test, please wait... This may take a while, thank you for your patience!"
            .into(),
    );
    let result = run_test(&auth_resp, &uploaded_files)?;
    println!("{}", result);
    sp.stop_with_message("Test finished".into());
    let body = AnalyticBody {
        files: uploaded_files,
        submission_id: auth_resp.submission_id().to_owned(),
        result,
    };
    Ok(body)
}

pub struct Config {
    pub test_set_id: String,
    pub phone: String,
    pub paths: Vec<PathBuf>,
}

impl Config {
    pub fn build(
        test_set_id: &String,
        phone: &String,
        files: Vec<&String>,
    ) -> Result<Self, io::Error> {
        let paths_clone = files
            .into_iter()
            .map(|x| -> PathBuf {
                let dir: PathBuf = std::env::current_dir().unwrap();
                dir.join(x)
            })
            .collect::<Vec<PathBuf>>();
        Ok(Self {
            test_set_id: test_set_id.into(),
            phone: phone.into(),
            paths: paths_clone,
        })
    }
}

#[derive(Debug, Serialize)]
struct TestRequestBody {
    files: Vec<FileDescription>,
    test_env: Vec<FileDescription>,
    test_entry: String,
}

#[tokio::main]
async fn run_test(
    params: &AuthResp,
    files: &Vec<FileDescription>,
) -> Result<String, reqwest::Error> {
    let body = TestRequestBody {
        files: files.clone().to_vec(),
        test_entry: params.test_entry().into(),
        test_env: params.test_env().clone().to_vec(),
    };
    let client = reqwest::Client::new();
    let resp = client
        .post(params.runner_location())
        .json(&body)
        .send()
        .await?;
    let data = resp.text().await?;
    Ok(data)
}

#[derive(Debug, Serialize)]
pub struct AnalyticBody {
    files: Vec<FileDescription>,
    submission_id: String,
    result: String,
}

#[tokio::main]
pub async fn send_analytic(body: AnalyticBody) -> Result<(), reqwest::Error> {
    let client = reqwest::Client::new();
    client
        .post(constants::ANALYTIC_ENDPOINT)
        .json(&body)
        .send()
        .await?;
    Ok(())
}
