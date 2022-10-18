#![allow(non_snake_case)]
use crate::constants;
use crate::files::FileDescription;
use crate::Config;
use mac_address;
use serde::{Deserialize, Serialize};
use std::{
    error, fmt,
    fmt::{Display, Formatter},
};

#[derive(Serialize, Deserialize, Debug)]
pub struct AuthResp {
    message: Option<String>,
    SubmissionId: Option<String>,
    Bucket: Option<String>,
    Dir: Option<String>,
    OSSAccessKeyId: Option<String>,
    Policy: Option<String>,
    Signature: Option<String>,
    RunnerLocation: Option<String>,
    TestEntry: Option<String>,
    TestEnv: Option<Vec<FileDescription>>,
}
impl AuthResp {
    pub fn failed(&self) -> bool {
        if let Some(_) = &self.SubmissionId {
            return false;
        };
        true
    }
    pub fn message(&self) -> &String {
        self.message.as_ref().unwrap()
    }
    pub fn submission_id(&self) -> &String {
        self.SubmissionId.as_ref().unwrap()
    }
    pub fn bucket(&self) -> &String {
        self.Bucket.as_ref().unwrap()
    }
    pub fn dir(&self) -> &String {
        self.Dir.as_ref().unwrap()
    }
    pub fn oss_access_key_id(&self) -> &String {
        self.OSSAccessKeyId.as_ref().unwrap()
    }
    pub fn policy(&self) -> &String {
        self.Policy.as_ref().unwrap()
    }
    pub fn signature(&self) -> &String {
        self.Signature.as_ref().unwrap()
    }
    pub fn runner_location(&self) -> &String {
        self.RunnerLocation.as_ref().unwrap()
    }
    pub fn test_entry(&self) -> &String {
        self.TestEntry.as_ref().unwrap()
    }
    pub fn test_env(&self) -> &Vec<FileDescription> {
        self.TestEnv.as_ref().unwrap()
    }
}

#[derive(Debug)]
pub struct AuthError {
    reason: String,
}
impl AuthError {
    pub fn new(reason: String) -> Self {
        Self { reason }
    }
}
impl Display for AuthError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", &self.reason)
    }
}
impl error::Error for AuthError {}

#[tokio::main]
pub async fn authenticate(config: &Config) -> Result<AuthResp, reqwest::Error> {
    let mac = match mac_address::get_mac_address() {
        Ok(option) => match option {
            Some(value) => value.to_string(),
            None => String::from("unknown"),
        },
        Err(_) => String::from("unknown"),
    };
    let query_string = format!(
        "setId={}&phone={}&mac={}",
        &config.test_set_id,
        &config.phone,
        urlencoding::encode(&mac)
    );
    let url = format!("{}?{}", constants::AUTH_ENDPOINT, query_string);
    let client = reqwest::Client::new();
    let resp = client.get(url.to_string()).send().await?;
    let auth_resp = resp.json::<AuthResp>().await?;
    Ok(auth_resp)
}
