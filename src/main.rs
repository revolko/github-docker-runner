use std::{collections::HashMap, env, error::Error};

use dotenvy::dotenv;
use reqwest::{
    header::{ACCEPT, AUTHORIZATION, USER_AGENT},
    Client, StatusCode,
};
use serde::Deserialize;

const GITHUB_API_URL: &str = "https://api.github.com";
const GITHUB_URL: &str = "https://github.com";
const MY_USER_AGENT: &str = "docker-runner";

#[derive(Deserialize)]
struct JIRToken {
    token: String,
    expires_at: String,
}

async fn get_jit_runner_token(
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
struct RunnerRegistration {
    token: String,
    token_schema: String,
    url: String,
}

async fn get_tenant_creds(
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

async fn get_runners(
    api_client: &Client,
    github_pat: &str,
    repo_name: &str,
) -> Result<String, Box<dyn Error>> {
    let resp = api_client
        .get(format!(
            "{GITHUB_API_URL}/repos/{repo_name}/actions/runners"
        ))
        .header(USER_AGENT, MY_USER_AGENT)
        .header(ACCEPT, "application/vnd.github+json")
        .header(AUTHORIZATION, format!("Bearer {github_pat}"))
        .send()
        .await?
        .text()
        .await?;

    return Ok(resp);
}

/// I cannot figure out what the VSSConnection is....
async fn test_connection(
    api_client: &Client,
    registraion: &RunnerRegistration,
) -> Result<StatusCode, Box<dyn Error>> {
    println!("request to {}", &registraion.url);
    let resp = api_client
        .get(&registraion.url)
        //.header(USER_AGENT, MY_USER_AGENT)
        //.header(ACCEPT, "application/vnd.github+json")
        .header(AUTHORIZATION, format!("Basic {}", registraion.token))
        .send()
        .await?
        .status();
    return Ok(resp);
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv().ok();
    let github_pat =
        env::var("GITHUB_FINE_GRAINED").expect("GitHub Fine Personal Access Token is not set.");
    let api_client = Client::new();
    let jit_token =
        get_jit_runner_token(&api_client, &github_pat, "revolko/github-docker-runner").await?;
    println!("{}", jit_token.token);

    let resp = get_tenant_creds(
        &api_client,
        &jit_token.token,
        "revolko/github-docker-runner",
    )
    .await?;
    println!("{:?}", resp);

    let resp = get_runners(&api_client, &github_pat, "revolko/github-docker-runner").await?;
    println!("{:?}", resp);

    return Ok(());
}
