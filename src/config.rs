use crate::{GITHUB_API_URL, GITHUB_URL, MY_USER_AGENT};
use reqwest::{
    header::{ACCEPT, AUTHORIZATION, USER_AGENT},
    Client,
};
use serde::Deserialize;
use std::{collections::HashMap, error::Error};

#[derive(Deserialize)]
pub struct JIRToken {
    pub token: String,
    expires_at: String,
}

pub async fn get_jit_runner_token(
    api_client: &Client,
    github_pat: &str,
    repo_name: &str,
) -> Result<JIRToken, Box<dyn Error>> {
    let resp = api_client
        .post(format!(
            "{GITHUB_API_URL}/repos/{repo_name}/actions/runners/registration-token"
        ))
        .header(USER_AGENT, MY_USER_AGENT)
        .header(ACCEPT, "application/vnd.github+json")
        .header(AUTHORIZATION, format!("Bearer {github_pat}"))
        .send()
        .await?
        .text()
        .await?;

    let jir_token: JIRToken = serde_json::from_str(&resp)?;
    return Ok(jir_token);
}

#[derive(Deserialize, Debug)]
pub struct RunnerRegistration {
    pub token: String,
    pub token_schema: String,
    pub url: String,
}

pub async fn get_tenant_creds(
    api_client: &Client,
    jit_token: &str,
    repo_name: &str,
) -> Result<RunnerRegistration, Box<dyn Error>> {
    let mut req_body = HashMap::new();
    req_body.insert("url", format!("{GITHUB_URL}/{repo_name}"));
    req_body.insert("runner_event", "register".to_string());

    let resp = api_client
        .post(format!("{GITHUB_API_URL}/actions/runner-registration"))
        .header(USER_AGENT, MY_USER_AGENT)
        .header(ACCEPT, "application/vnd.github+json")
        .header(AUTHORIZATION, format!("RemoteAuth {jit_token}"))
        .json(&req_body)
        .send()
        .await?
        .text()
        .await?;

    let registration: RunnerRegistration = serde_json::from_str(&resp)?;

    return Ok(registration);
}

// I cannot figure out what the VSSConnection is....
//async fn test_connection(
//    api_client: &Client,
//    registraion: &RunnerRegistration,
//) -> Result<StatusCode, Box<dyn Error>> {
//    println!("request to {}", &registraion.url);
//    let resp = api_client
//        .get(&registraion.url)
//        //.header(USER_AGENT, MY_USER_AGENT)
//        //.header(ACCEPT, "application/vnd.github+json")
//        .header(AUTHORIZATION, format!("Basic {}", registraion.token))
//        .send()
//        .await?
//        .status();
//    return Ok(resp);
//}
