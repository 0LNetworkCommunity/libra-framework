use diem_github_client::{Client, Error};
use serde::Deserialize;
use serde_json::json;

pub trait LibraGithubClient {
    fn make_genesis_pull_request(
        &self,
        repo_owner: &str,
        repo_name: &str,
        github_username: &str,
        branch: Option<&str>,
    ) -> Result<(), Error>;
    fn fork_genesis_repo(&self, repo_owner: &str, repo_name: &str) -> Result<(), Error>;
    fn get_authenticated_user(&self) -> Result<String, Error>;
}

impl LibraGithubClient for Client {
    fn make_genesis_pull_request(
        &self,
        genesis_repo_owner: &str,
        genesis_repo_name: &str,
        pull_username: &str,
        branch: Option<&str>,
    ) -> Result<(), Error> {
        let branch = branch.unwrap_or("main");
        let head = format!("{}:{}", pull_username, branch);
        let json = json!({"head": &head, "base": &branch, "title": pull_username});
        let api_path = format!(
            "https://api.github.com/repos/{}/{}/pulls",
            genesis_repo_owner, genesis_repo_name
        );

        let resp = self.upgrade_request(ureq::post(&api_path)).send_json(json);

        match resp.status() {
            200 => Ok(()),
            201 => Ok(()),
            _ => Err(resp.into()),
        }
    }
    fn fork_genesis_repo(
        &self,
        genesis_repo_owner: &str,
        genesis_repo_name: &str,
    ) -> Result<(), Error> {
        let json = json!({});

        let api_path = format!(
            "https://api.github.com/repos/{}/{}/forks",
            genesis_repo_owner, genesis_repo_name
        );
        let resp = self.upgrade_request(ureq::post(&api_path)).send_json(json);

        match resp.status() {
            200 => Ok(()),
            201 => Ok(()),
            202 => Ok(()),
            _ => Err(resp.into()),
        }
    }

    fn get_authenticated_user(&self) -> Result<String, Error> {
        let api_path = "https://api.github.com/user";

        let resp = self.upgrade_request(ureq::get(api_path)).call();
        #[derive(Deserialize)]
        struct Test {
            login: String,
        }

        match resp.status() {
            200 => {
                let d: Test = resp.into_json_deserialize().unwrap();
                Ok(d.login)
            }
            _ => Err(resp.into()),
        }
    }
}
