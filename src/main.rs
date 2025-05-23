use std::{env, error::Error};

use dotenvy::dotenv;
use reqwest::{
    header::{ACCEPT, AUTHORIZATION, USER_AGENT},
    Client,
};

mod config;

use config::{get_jit_runner_token, get_tenant_creds};

static GITHUB_API_URL: &str = "https://api.github.com";
static GITHUB_URL: &str = "https://github.com";
static MY_USER_AGENT: &str = "docker-runner";

async fn register_runner(
    api_client: &Client,
    token: &str,
    repo_name: &str,
) -> Result<String, Box<dyn Error>> {
    let resp = api_client
        .get(format!(
            "{GITHUB_API_URL}/repos/{repo_name}/actions/runners/register"
        ))
        .header(USER_AGENT, MY_USER_AGENT)
        .header(ACCEPT, "application/vnd.github+json")
        .header(AUTHORIZATION, format!("RemoteAuth {token}"))
        .send()
        .await?
        .text()
        .await?;

    return Ok(resp);
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv().ok();
    let github_pat =
        env::var("GITHUB_FINE_GRAINED").expect("GitHub Fine Personal Access Token is not set.");
    let api_client = Client::new();
    let repo_name = "revolko/github-docker-runner";
    let jit_token = get_jit_runner_token(&api_client, &github_pat, repo_name).await?;
    println!("{}", jit_token.token);

    let resp = get_tenant_creds(
        &api_client,
        &jit_token.token,
        "revolko/github-docker-runner",
    )
    .await?;
    println!("{:?}", resp);

    let resp = register_runner(&api_client, &jit_token.token, repo_name).await?;
    println!("{:?}", resp);

    return Ok(());
}
