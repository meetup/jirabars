// Third party
use reqwest::{Client, Error};

#[derive(Deserialize, Debug)]
pub struct Payload {
    pub action: String,
    pub number: usize,
    pub pull_request: PullRequest,
}

impl Payload {
    /// return true if we should attempt to update this
    /// pull request
    pub fn updatable(&self) -> bool {
        "opened" == self.action
    }

    /// url for updating pull request
    pub fn pull_url(&self) -> &String {
        &self.pull_request.url
    }
}

#[derive(Deserialize, Debug)]
pub struct PullRequest {
    pub url: String,
    pub html_url: String,
    pub title: String,
    pub state: String,
    pub body: Option<String>,
    pub head: Ref,
}

#[derive(Deserialize, Debug)]
pub struct Ref {
    #[serde(rename = "ref")]
    pub branch: String,
    pub repo: Repository,
}

#[derive(Deserialize, Debug)]
pub struct Repository {
    /// {owner}/{repo}
    pub full_name: String,
}

pub fn patch(token: &String, url: &String, body: &String) -> Result<(), Error> {
    Client::new()
        .expect("failed to initialize client")
        .patch(url)
        .expect("failed to parse url")
        .basic_auth("", Some(token.clone()))
        .json(&json!({ "body": body }))
        .expect("failed to encode body")
        .send()
        .map(|_| ())
}
