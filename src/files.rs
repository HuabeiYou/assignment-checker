use crate::authentication::AuthResp;
use reqwest::multipart;
use serde::{Deserialize, Serialize};
use std::{error, fs, io, path::PathBuf};

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct FileDescription {
    pub key: String,
    pub bucket: String,
}

#[derive(Debug)]
pub struct UploadJob {
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
    pub fn build(params: &AuthResp, path: PathBuf) -> Result<Self, Box<dyn error::Error>> {
        let file_name = String::from(path.file_name().unwrap().to_str().unwrap());
        let key = format!("{}/{}", params.dir(), file_name);
        let data = fs::read(path).unwrap();
        Ok(Self {
            key,
            bucket: params.bucket().into(),
            destination: format!("https://{}.oss-accelerate.aliyuncs.com", params.bucket()),
            oss_access_key: params.oss_access_key_id().into(),
            policy: params.policy().into(),
            signature: params.signature().into(),
            file_name,
            data,
        })
    }
}

pub fn validate_files(paths: &Vec<PathBuf>) -> Result<(), io::Error> {
    for file in paths {
        let is_file = file.is_file();
        if !is_file {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                "File not found. Please double check the file path.",
            ));
        }
        let file_size = file.metadata()?.len();
        if file_size > 1024 * 1000 {
            return Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "File too large. Text file is unlikely to exceed 1MB.",
            ));
        }
    }
    Ok(())
}

#[tokio::main]
pub async fn upload(jobs: Vec<UploadJob>) -> Result<Vec<FileDescription>, Box<dyn error::Error>> {
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
        match resp.error_for_status() {
            Ok(_) => results.push(FileDescription {
                key: job.key,
                bucket: job.bucket,
            }),
            Err(_) => return Err(Box::from("Impeded communication, please try again later.")),
        }
    }
    Ok(results)
}
