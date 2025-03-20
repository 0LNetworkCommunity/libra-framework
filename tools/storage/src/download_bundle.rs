use anyhow::{Context, Result};
use reqwest;
use serde::Deserialize;
use std::fs;
use std::path::Path;

#[derive(Deserialize)]
struct GitHubContent {
    download_url: Option<String>,
    #[serde(rename = "type")]
    content_type: String,
    name: String,
}

pub async fn download_github_folder(
    owner: &str,
    repo: &str,
    path: &str,
    branch: &str,
    output_dir: &str,
) -> Result<()> {
    let client = reqwest::Client::new();
    let mut pending_dirs = vec![(path.to_string(), output_dir.to_string())];

    while let Some((current_path, current_dir)) = pending_dirs.pop() {
        let api_url = format!(
            "https://api.github.com/repos/{}/{}/contents/{}?ref={}",
            owner, repo, current_path, branch
        );

        println!("Downloading from: {}", api_url);

        let contents: Vec<GitHubContent> = client
            .get(&api_url)
            .header("User-Agent", "libra-framework-downloader")
            .send()
            .await
            .context("Failed to send request")?
            .json()
            .await
            .context("Failed to parse JSON response")?;

        fs::create_dir_all(&current_dir)?;

        for item in contents {
            let output_path = Path::new(&current_dir).join(&item.name);

            if item.content_type == "file" {
                if let Some(download_url) = item.download_url {
                    println!("Downloading file: {}", item.name);
                    let content = client
                        .get(&download_url)
                        .header("User-Agent", "libra-framework-downloader")
                        .send()
                        .await?
                        .bytes()
                        .await?;

                    fs::write(&output_path, content)
                        .with_context(|| format!("Failed to write file: {}", item.name))?;
                }
            } else if item.content_type == "dir" {
                println!("Processing directory: {}", item.name);
                let new_path = format!("{}/{}", current_path, item.name);
                pending_dirs.push((
                    new_path,
                    output_path.to_str().unwrap().to_string(),
                ));
            }
        }
    }

    Ok(())
}
