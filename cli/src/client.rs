use anyhow::{Context, Result};
use std::collections::HashMap;

use crate::config::Config;

pub async fn fetch_env(cfg: &Config, project: &str, env: &str) -> Result<HashMap<String, String>> {
    let url = format!(
        "{}/api/projects/{}/env?environment={}",
        cfg.server_url.trim_end_matches('/'),
        project,
        env
    );

    let client = reqwest::Client::new();
    let resp = client
        .get(&url)
        .bearer_auth(&cfg.token)
        .send()
        .await
        .context("failed to connect to server")?;

    if resp.status() == 401 {
        anyhow::bail!("Unauthorized — check your token");
    }
    if resp.status() == 404 {
        anyhow::bail!("Project '{}' or environment '{}' not found", project, env);
    }
    if !resp.status().is_success() {
        anyhow::bail!("Server error: {}", resp.status());
    }

    let vars: HashMap<String, String> = resp.json().await.context("failed to parse response")?;
    Ok(vars)
}
