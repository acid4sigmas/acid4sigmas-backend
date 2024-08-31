use std::collections::HashMap;


use anyhow::Result;

use super::RepoInfo;
use super::Repo;


pub async fn get_repo_info(owner: &str, repo: &str) -> Result<RepoInfo> {
    let url = format!(
        "https://api.github.com/repos/{}/{}",
        owner, repo
    );

    let client = reqwest::Client::new();
    let response = client
        .get(&url)
        .header("User-Agent", "reqwest")
        .send()
        .await;

    match response {
        Ok(resp) => {
            if resp.status().is_success() {
                let repo_: Repo = resp.json().await?;
                
                let repo_langs = get_repo_language(owner, repo).await?;

                let repo_info = RepoInfo { repo: repo_, languages: repo_langs };

                return Ok(repo_info)

            } else {
                return Err(anyhow::anyhow!(format!("github returned: {}", resp.status())))
            }
        }
        Err(e) => return Err(e.into())
    }

}



async fn get_repo_language(owner: &str, repo: &str) -> Result<Option<HashMap<String, u64>>> {
    let url = format!(
        "https://api.github.com/repos/{}/{}/languages",
        owner, repo
    );

    let client = reqwest::Client::new();
    let response = client
        .get(&url)
        .header("User-Agent", "reqwest")
        .send()
        .await?;

    let languages = response.json::<HashMap<String, u64>>().await?;
    if languages.is_empty() {
        Ok(None)
    } else {
        Ok(Some(languages))
    }
}
